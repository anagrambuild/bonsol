use bonsol_schema::{
    parse_ix_data, ChannelInstruction, ChannelInstructionIxType, ExecutionInputType, ExecutionRequestV1, StatusV1
};
use anyhow::Result;
use bincode::{self, config, config::Configuration, Decode, Encode};
use bonsai_sdk::alpha::Client;
use redb::{ReadableTable, RedbValue, TableDefinition};
use risc0_zkvm::compute_image_id;
use serde::Serialize;
use serde_json::to_vec;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::{Duration, Instant, UNIX_EPOCH};
use std::{collections::HashMap, fs};
use std::{mem, time};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{Mutex, Semaphore};
use tokio::task::{JoinHandle, JoinSet};

use crate::ingest::BonsolInstruction;

#[derive(Debug, Encode, Decode)]
#[repr(u8)]
pub enum ExecutionRequestState {
    Unknown = 0,
    Queued,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Encode, Decode)]
pub struct ExecutionRequestStatus {
    pub request_id: String,
    pub proving_session_id: String,
    pub status: ExecutionRequestState,
    pub result: Option<Vec<u8>>,
    pub timestamp: u64,
}

pub struct BonsaiRunner {
    loaded_images: Arc<HashMap<String, ()>>,
    client: Arc<Client>,
    worker_handle: Option<JoinHandle<Result<()>>>,
    data_dir: String,
    db_handle: Option<JoinHandle<Result<()>>>,
}

const ERS_TABLE: TableDefinition<&str, (u64, &[u8])> =
    TableDefinition::new("execution_request_status");
const config: Configuration = config::standard();
pub type ExecutionRequestChannel = UnboundedSender<BonsolInstruction>;

fn image_id(image: &[u8]) -> String {
    hex::encode(compute_image_id(image).unwrap())
}

fn parse_image_id<'a>(raw_image_id: &'a [u8]) -> Result<&'a str> {
    let ii = from_utf8(raw_image_id)?;
    Ok(ii)
}

async fn get_input_data<'a>(input: ExecutionRequestV1<'a>) -> Result<&'a [u8]> {
    if let Some(data) = input.input_data() {
        return match input.input_type() {
            ExecutionInputType::DATA => Ok(data.bytes()),
            ExecutionInputType::URL => {
                //not implemented
                Err(anyhow::anyhow!("URL input type not implemented"))
            }
            _ => Err(anyhow::anyhow!("Invalid input type")),
        };
    }
    Err(anyhow::anyhow!("Invalid input type"))
}

impl BonsaiRunner {
    pub fn new(
        client: Client,
        image_dir: String,
        data_dir: String,
    ) -> Result<BonsaiRunner> {
        let dir = fs::read_dir(image_dir)?;
        let mut loaded_images: HashMap<String, ()> = HashMap::new();
        for entry in dir {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let bytes = fs::read(entry.path())?;
                let image_id = hex::encode(compute_image_id(&bytes)?);
                client.upload_img(image_id.as_str(), bytes)?;
                println!("Loaded image: {}", &image_id);
                loaded_images.insert(image_id, ());
                
            }
        }

        Ok(BonsaiRunner {
            loaded_images: Arc::new(loaded_images),
            client: Arc::new(client),
            worker_handle: None,
            db_handle: None,
            data_dir,
        })
    }

    pub fn start<'a>(&mut self) -> Result<ExecutionRequestChannel> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<BonsolInstruction>();
        let (dbtx, dbrx) = tokio::sync::mpsc::unbounded_channel::<ExecutionRequestStatus>();
        let client = self.client.clone();
        let loaded_images = self.loaded_images.clone();
        let data_dir = self.data_dir.clone();
        let db = Arc::new(redb::Database::create(data_dir)?);
        let txn = db.begin_write()?;
        txn.open_table(ERS_TABLE)?;
        txn.commit()?;
        self.db_handle = Some(tokio::task::spawn_blocking(move || {
            let db = db.clone();
            let mut dbrx = dbrx;
            while let Some(ers) = dbrx.blocking_recv() {
                
                let req = ers.request_id.clone();
                let mut created_at = time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)?
                    .as_secs();
                let read = db.begin_read()?;
                
                let table = read.open_table(ERS_TABLE);
                if table.is_err() {
                    println!("Error opening table for read: {:?}", table.err());
                    continue;
                }
                
                if let Some(exising) = table.unwrap().get(req.as_str())? {
                    let (timestamp, _) = exising.value();
                    created_at = timestamp;
                }
                
                let txn = db.begin_write()?;
                {
                    let mut table = txn.open_table(ERS_TABLE)?;
                    let bytes = bincode::encode_to_vec(ers, config)?; // Use the cloned `ers`
                    table.insert(req.as_str(), (created_at, &*bytes))?;
                }
                txn.commit()?;
            }
            Ok(())
        }));
        self.worker_handle = Some(tokio::spawn(async move {
            let mut rx = rx;
            let sem = Semaphore::new(15);
            while let Some(bix) = rx.recv().await {    
                let _permit = sem.acquire().await.unwrap();
                let dbtx = dbtx.clone();
                let client = client.clone();
                let loaded_images = loaded_images.clone();
                let _: JoinHandle<Result<()>> = tokio::spawn(async move {
                    if let Ok(bonsol_ix_type) = parse_ix_data(&bix.data) {
                        match bonsol_ix_type.ix_type() {
                            ChannelInstructionIxType::ExecuteV1 => {
                                if let Some(variant) = bonsol_ix_type.execute_v1_nested_flatbuffer() {
                                    let input = to_vec(get_input_data(variant).await?)?;
                                    let image_id = parse_image_id(
                                        variant.image_id().map(|g| g.bytes()).unwrap_or(&[]),
                                    )?;
                                    if loaded_images.contains_key(image_id) {
                                        let uuid = client.upload_input(input)?;
                                        let assumptions: Vec<String> = vec![];
                                        let session = client.create_session(
                                            image_id.to_string(),
                                            uuid,
                                            assumptions,
                                        )?;
                                        let exec_id =
                                            from_utf8(variant.execution_id().unwrap().bytes())?;

                                        let ers = ExecutionRequestStatus {
                                            request_id: exec_id.to_string(),
                                            proving_session_id: session.uuid.clone(),
                                            status: ExecutionRequestState::Queued,
                                            result: None,
                                            timestamp: time::SystemTime::now()
                                                .duration_since(UNIX_EPOCH)?
                                                .as_secs(),
                                        };
                                        dbtx.send(ers).unwrap();
                                        println!("Session created: {}", session.uuid);
                                        let mut status = "";
                                        while status != "SUCCEEDED" && status != "FAILED" {
                                            let res = session.status(&client)?;
                                            if res.status == "RUNNING" {
                                                eprintln!(
                                                    "Current status: {} - state: {} - continue polling...",
                                                    res.status,
                                                    res.state.unwrap_or_default()
                                                );
                                                let ers = ExecutionRequestStatus {
                                                    request_id: exec_id.to_string(),
                                                    proving_session_id: session.uuid.clone(),
                                                    status: ExecutionRequestState::InProgress,
                                                    result: None,
                                                    timestamp: time::SystemTime::now()
                                                        .duration_since(UNIX_EPOCH)?
                                                        .as_secs(),
                                                };
                                                dbtx.send(ers).unwrap();
                                                tokio::time::sleep(Duration::from_secs(5)).await;
                                                continue;
                                            }
                                            if res.status == "SUCCEEDED" {
                                                // Download the receipt, containing the output

                                                let receipt_url = res
                                                    .receipt_url
                                                    .expect("API error, missing receipt on completed session");

                                                let receipt_buf = client.download(&receipt_url)?;
                                                let ers = ExecutionRequestStatus {
                                                    request_id: exec_id.to_string(),
                                                    proving_session_id: session.uuid.clone(),
                                                    status: ExecutionRequestState::Completed,
                                                    result: Some(receipt_buf),
                                                    timestamp: time::SystemTime::now()
                                                        .duration_since(UNIX_EPOCH)?
                                                        .as_secs(),
                                                };
                                                eprintln!("Session completed: {:?}", status);
                                                dbtx.send(ers).unwrap();
                                                status = "SUCCEEDED";
                                                break;
                                            } else {
                                                let ers = ExecutionRequestStatus {
                                                    request_id: exec_id.to_string(),
                                                    proving_session_id: session.uuid.clone(),
                                                    status: ExecutionRequestState::Failed,
                                                    result: None,
                                                    timestamp: time::SystemTime::now()
                                                        .duration_since(UNIX_EPOCH)?
                                                        .as_secs(),
                                                };
                                                eprintln!("Session failed: {:?}", status);
                                                status = "FAILED";
                                                dbtx.send(ers).unwrap();
                                                break;
                                            }
                                        }

                                        if status == "SUCCEEDED" {
                                            let snark =
                                                client.create_snark(session.uuid.clone())?;
                                            eprint!("Creating snark... {:?}", snark);
                                            let mut status = "";
                                            while status != "SUCCEEDED" && status != "FAILED" {
                                                let res = snark.status(&client)?;
                                                if res.status == "RUNNING" {
                                                    eprintln!(
                                                        "Current status: {} - continue polling...",
                                                        res.status,
                                                    );
                                                    tokio::time::sleep(Duration::from_secs(5))
                                                        .await;
                                                    continue;
                                                }
                                                if res.status == "SUCCEEDED" && res.output.is_some() {
                                                    
                                                    // Download the receipt, containing the output
                                                    if let Some(sr) = res.output {
                                                        println!("Receipt: {:?}", sr.snark);
                                                        let ers = ExecutionRequestStatus {
                                                            request_id: exec_id.to_string(),
                                                            proving_session_id: session
                                                                .uuid
                                                                .clone(),
                                                            status:
                                                                ExecutionRequestState::Completed,
                                                            result: None,
                                                            timestamp: time::SystemTime::now()
                                                                .duration_since(UNIX_EPOCH)?
                                                                .as_secs(),
                                                        };
                                                        dbtx.send(ers).unwrap();
                                                        break;
                                                    }
                                                } else {
                                                    let ers = ExecutionRequestStatus {
                                                        request_id: exec_id.to_string(),
                                                        proving_session_id: session.uuid.clone(),
                                                        status: ExecutionRequestState::Failed,
                                                        result: None,
                                                        timestamp: time::SystemTime::now()
                                                            .duration_since(UNIX_EPOCH)?
                                                            .as_secs(),
                                                    };
                                                    eprintln!("Session failed: {:?}", status);
                                                    status = "FAILED";
                                                    dbtx.send(ers).unwrap();
                                                    break;
                                                }
                                            }
                                        }
                                    } else {
                                        eprintln!("Image not found");
                                    }
                                }
                            }

                           
                            _ => {}
                        }
                    }
                    Ok(())
                });
            }
            Ok(())
        }));
        Ok(tx)
    }

    pub fn stop(&mut self) -> Result<()> {
        self.worker_handle.take().unwrap().abort();
        Ok(())
    }
}
