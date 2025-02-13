mod utils;
pub mod verify_prover_version;

use crate::transaction_sender::TransactionStatus;

use {
    solana_sdk::instruction::AccountMeta,
    utils::{check_stark_compression_tools_path, check_x86_64arch},
};

use {
    crate::{
        config::ProverNodeConfig,
        observe::*,
        risc0_runner::utils::async_to_json,
        transaction_sender::{RpcTransactionSender, TransactionSender},
        MissingImageStrategy,
    },
    bonsol_interface::{
        bonsol_schema::{ClaimV1, DeployV1, ExecutionRequestV1},
        prover_version::{ProverVersion, VERSION_V1_2_1},
    },
    dashmap::DashMap,
    risc0_binfmt::MemoryImage,
    risc0_zkvm::{ExitCode, Journal, SuccinctReceipt},
    solana_sdk::{pubkey::Pubkey, signature::Signature},
    std::{
        convert::TryInto, env::consts::ARCH, fs, io::Cursor, path::Path, sync::Arc, time::Duration,
    },
};

use {
    crate::types::{BonsolInstruction, ProgramExec},
    anyhow::Result,
    bonsol_interface::bonsol_schema::{parse_ix_data, root_as_deploy_v1, ChannelInstructionIxType},
    bonsol_prover::{
        image::Image,
        input_resolver::{InputResolver, ProgramInput},
        prover::{get_risc0_prover, new_risc0_exec_env},
        util::get_body_max_size,
    },
    risc0_groth16::{ProofJson, Seal},
    risc0_zkvm::{
        recursion::identity_p254,
        sha::{Digest, Digestible},
        InnerReceipt, MaybePruned, ReceiptClaim, VerifierContext,
    },
    tempfile::tempdir,
    thiserror::Error,
    tokio::{
        fs::File, io::AsyncReadExt, process::Command, sync::mpsc::UnboundedSender, task::JoinHandle,
    },
    tracing::{error, info, warn},
    verify_prover_version::verify_prover_version,
};

const REQUIRED_PROVER: ProverVersion = VERSION_V1_2_1;

#[derive(Debug, Error)]
pub enum Risc0RunnerError {
    #[error("Empty instruction")]
    EmptyInstruction,
    #[error("Invalid data")]
    InvalidData,
    #[error("Img too large")]
    ImgTooLarge,
    #[error("Img load error")]
    ImgLoadError,
    #[error("Image Data Unavailable")]
    ImageDataUnavailable,
    #[error("Image download error")]
    ImageDownloadError(#[from] anyhow::Error),
    #[error("Transaction error")]
    TransactionError(String),
    #[error("Error with proof compression")]
    ProofCompressionError,
    #[error("Error with proof generation")]
    ProofGenerationError,
    #[error("Invalid prover version {0}, expected {1}")]
    InvalidProverVersion(ProverVersion, ProverVersion),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimStatus {
    Claiming,
    Submitted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InflightProof {
    pub execution_id: String,
    pub image_id: String,
    pub status: ClaimStatus,
    pub claim_signature: Signature,
    pub submission_signature: Option<Signature>,
    pub expiry: u64,
    pub requester: Pubkey,
    pub program_callback: Option<ProgramExec>,
    pub additional_accounts: Vec<AccountMeta>,
}

type InflightProofs = Arc<DashMap<String, InflightProof>>;
type InflightProofRef<'a> = &'a DashMap<String, InflightProof>;

type LoadedImageMap = Arc<DashMap<String, Image>>;
type LoadedImageMapRef<'a> = &'a DashMap<String, Image>;

type InputStagingArea = Arc<DashMap<String, Vec<ProgramInput>>>;
type InputStagingAreaRef<'a> = &'a DashMap<String, Vec<ProgramInput>>;

pub struct Risc0Runner {
    config: Arc<ProverNodeConfig>,
    loaded_images: LoadedImageMap,
    worker_handle: Option<JoinHandle<Result<()>>>,
    inflight_proof_worker_handle: Option<JoinHandle<Result<()>>>,
    txn_sender: Arc<RpcTransactionSender>,
    input_staging_area: InputStagingArea,
    self_identity: Arc<Pubkey>,
    inflight_proofs: InflightProofs,
    input_resolver: Arc<dyn InputResolver + 'static>,
}

impl Risc0Runner {
    pub async fn new(
        config: ProverNodeConfig,
        self_identity: Pubkey,
        txn_sender: Arc<RpcTransactionSender>,
        input_resolver: Arc<dyn InputResolver + 'static>,
    ) -> Result<Risc0Runner> {
        if !check_x86_64arch() {
            warn!("Bonsol node will not compress STARKs to SNARKs after successful risc0vm\nproving due to stark compression tooling requiring x86_64 architectures - virtualization will also fail");
        }

        check_stark_compression_tools_path(&config.stark_compression_tools_path)?;

        if !std::path::Path::new(&config.risc0_image_folder).exists() {
            fs::create_dir_all(&config.risc0_image_folder)?;
        }

        let dir = fs::read_dir(&config.risc0_image_folder)?;
        let loaded_images = DashMap::new();
        for entry in dir {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let img = Image::new(entry.path()).await?;
                info!("Loaded image: {}", &img.id);
                loaded_images.insert(img.id.clone(), img);
            }
        }

        Ok(Risc0Runner {
            config: Arc::new(config),
            loaded_images: Arc::new(loaded_images),
            worker_handle: None,
            inflight_proof_worker_handle: None,
            txn_sender,
            input_staging_area: Arc::new(DashMap::new()),
            self_identity: Arc::new(self_identity),
            inflight_proofs: Arc::new(DashMap::new()),
            input_resolver,
        })
    }

    // TODO: break up pipleine into smaller domains to make it easier to test
    // Break into Image handling, Input handling, Execution Request
    // Inputs and Image should be service used by this prover.
    pub fn start(&mut self) -> Result<UnboundedSender<BonsolInstruction>> {
        verify_prover_version(REQUIRED_PROVER)
            .expect("Bonsol build conflict: prover version is not supported");
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<BonsolInstruction>();
        let loaded_images = self.loaded_images.clone();
        // TODO: move image handling out of prover
        let img_client = Arc::new(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(
                    self.config.image_download_timeout_secs as u64,
                ))
                .build()?,
        );
        let config = self.config.clone();
        let self_id = self.self_identity.clone();
        let input_staging_area = self.input_staging_area.clone();
        let inflight_proofs = self.inflight_proofs.clone();
        let txn_sender = self.txn_sender.clone();
        self.inflight_proof_worker_handle = Some(tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let current_block = txn_sender.get_current_block().await.unwrap_or(0);
                inflight_proofs.retain(|_, v| {
                    if v.expiry < current_block {
                        emit_event!(MetricEvents::ProofExpired, execution_id => v.execution_id.clone());
                        return false;
                    }
                    match &v.status {
                        ClaimStatus::Claiming => {
                            let sig = v.claim_signature;
                            let inner_status = txn_sender.get_signature_status(&sig);
                            return match inner_status {
                                None => false,
                                Some(status) => {
                                    match status {
                                        TransactionStatus::Confirmed(status) => {
                                            txn_sender.clear_signature_status(&sig);
                                            if status.err.is_some() {
                                                info!("Claim Transaction Failed");

                                            }
                                            status.err.is_none()
                                        },
                                        _ => true
                                    }
                                }
                            };
                        }
                        ClaimStatus::Submitted => {
                            if let Some(sig) = v.submission_signature.as_ref() {
                                let inner_status = txn_sender.get_signature_status(sig);
                                return match inner_status {
                                    None => false,
                                    Some(status) => {
                                        match status {
                                            TransactionStatus::Confirmed(status) => {
                                                txn_sender.clear_signature_status(sig);
                                                if status.err.is_some() {
                                                    emit_event!(MetricEvents::ProofSubmissionError, sig => sig.to_string());
                                                }
                                                status.err.is_none()
                                            },
                                            _ => true
                                        }
                                    }
                                };
                            }
                        }
                    };
                    true
                });
            }
        }));

        let inflight_proofs = self.inflight_proofs.clone();
        let txn_sender = self.txn_sender.clone();
        let input_resolver = self.input_resolver.clone();
        self.worker_handle = Some(tokio::spawn(async move {
            while let Some(bix) = rx.recv().await {
                let txn_sender = txn_sender.clone();
                let loaded_images = loaded_images.clone();
                let config = config.clone();
                let img_client = img_client.clone();
                let input_resolver = input_resolver.clone();
                let self_id = self_id.clone();
                let input_staging_area = input_staging_area.clone();
                let inflight_proofs = inflight_proofs.clone();
                tokio::spawn(async move {
                    let bonsol_ix_type =
                        parse_ix_data(&bix.data).map_err(|_| Risc0RunnerError::InvalidData)?;
                    let result = match bonsol_ix_type.ix_type() {
                        ChannelInstructionIxType::DeployV1 => {
                            let payload = bonsol_ix_type
                                .deploy_v1_nested_flatbuffer()
                                .ok_or::<anyhow::Error>(
                                Risc0RunnerError::EmptyInstruction.into(),
                            )?;
                            emit_counter!(MetricEvents::ImageDeployment, 1, "image_id" => payload.image_id().unwrap_or_default());
                            handle_image_deployment(&config, &img_client, payload, &loaded_images)
                                .await
                        }
                        ChannelInstructionIxType::ExecuteV1 => {
                            info!("Received execution request");
                            // Evaluate the execution request and decide if it should be claimed
                            let payload = bonsol_ix_type
                                .execute_v1_nested_flatbuffer()
                                .ok_or::<anyhow::Error>(
                                Risc0RunnerError::EmptyInstruction.into(),
                            )?;
                            let er_prover_version: ProverVersion = payload
                                .prover_version()
                                .try_into()
                                .map_err::<anyhow::Error, _>(|_| {
                                    Risc0RunnerError::InvalidProverVersion(
                                        ProverVersion::UnsupportedVersion,
                                        REQUIRED_PROVER,
                                    )
                                    .into()
                                })?;
                            if er_prover_version != REQUIRED_PROVER {
                                return Err(Risc0RunnerError::InvalidProverVersion(
                                    er_prover_version,
                                    REQUIRED_PROVER,
                                )
                                .into());
                            }
                            handle_execution_request(
                                &config,
                                &inflight_proofs,
                                input_resolver.clone(),
                                img_client.clone(),
                                &txn_sender,
                                &loaded_images,
                                &input_staging_area,
                                bix.last_known_block,
                                payload,
                                &bix.accounts,
                            )
                            .await
                        }
                        ChannelInstructionIxType::ClaimV1 => {
                            info!("Claim Event");
                            let payload = bonsol_ix_type
                                .claim_v1_nested_flatbuffer()
                                .ok_or::<anyhow::Error>(
                                Risc0RunnerError::EmptyInstruction.into(),
                            )?;
                            handle_claim(
                                &config,
                                &self_id,
                                &inflight_proofs,
                                input_resolver.clone(),
                                &txn_sender,
                                &loaded_images,
                                &input_staging_area,
                                payload,
                                &bix.accounts,
                            )
                            .await
                        }
                        ChannelInstructionIxType::StatusV1 => Ok(()),
                        _ => {
                            info!("Unknown instruction type");
                            Ok(())
                        }
                    };
                    if result.is_err() {
                        info!("Error: {:?}", result);
                    }
                    result
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

pub async fn handle_claim<'a>(
    config: &ProverNodeConfig,
    self_identity: &Pubkey,
    in_flight_proofs: InflightProofRef<'a>,
    input_resolver: Arc<dyn InputResolver + 'static>,
    transaction_sender: &RpcTransactionSender,
    loaded_images: LoadedImageMapRef<'a>,
    input_staging_area: InputStagingAreaRef<'a>,
    claim: ClaimV1<'a>,
    accounts: &[Pubkey], // need to create cannonical parsing of accounts per instruction type for my flatbuffer model or use shank
) -> Result<()> {
    info!("Received claim event");
    let claimer = accounts[3];
    let execution_id = claim.execution_id().ok_or(Risc0RunnerError::InvalidData)?;
    if &claimer != self_identity {
        let attempt = in_flight_proofs.remove(execution_id);
        if let Some((ifp, claim)) = attempt {
            if let ClaimStatus::Claiming = claim.status {
                transaction_sender.clear_signature_status(&claim.claim_signature);
                emit_event!(MetricEvents::ClaimMissed, execution_id => ifp, signature => &claim.claim_signature.to_string());
            }
        }
        return Ok(());
    }

    let claim_status = in_flight_proofs
        .get(execution_id)
        .map(|v| v.value().to_owned());
    if let Some(mut claim) = claim_status {
        emit_event!(MetricEvents::ClaimReceived, execution_id => execution_id);
        if let ClaimStatus::Claiming = claim.status {
            if let Some(image) = loaded_images.get(&claim.image_id) {
                if image.data.is_none() {
                    return Err(Risc0RunnerError::ImageDataUnavailable.into());
                }
                //if image is not loaded at claim, fail
                let mut inputs = input_staging_area
                    .get(execution_id)
                    .ok_or(Risc0RunnerError::InvalidData)?
                    .value()
                    .clone(); //clone soe we dont hold a reference over http requests
                let unresolved_count = inputs
                    .iter()
                    .filter(|i| match i {
                        ProgramInput::Unresolved(_) => true,
                        _ => false,
                    })
                    .count();

                if unresolved_count > 0 {
                    info!("{} outstanding inputs", unresolved_count);

                    emit_event_with_duration!(MetricEvents::InputDownload, {
                        input_resolver.resolve_private_inputs(execution_id, &mut inputs, Arc::new(transaction_sender)).await?;
                    }, execution_id => execution_id, stage => "private");
                    input_staging_area.insert(execution_id.to_string(), inputs);
                    // one of the huge problems with the claim system is that we are not guaranteed to have
                    // the inputs we need at the time we claim and no way to
                }
                info!("{} inputs resolved", unresolved_count);

                // drain the inputs and own them here, this is a bit of a hack but it works
                let (eid, inputs) = input_staging_area
                    .remove(execution_id)
                    .ok_or(Risc0RunnerError::InvalidData)?;
                let mem_image = image.get_memory_image()?;
                let result: Result<
                    (Journal, Digest, SuccinctReceipt<ReceiptClaim>),
                    Risc0RunnerError,
                > = tokio::task::spawn_blocking(move || {
                    risc0_prove(mem_image, inputs).map_err(|e| {
                        info!("Error generating proof: {:?}", e);
                        Risc0RunnerError::ProofGenerationError
                    })
                })
                .await?;
                match result {
                    Ok((journal, assumptions_digest, reciept)) => {
                        let compressed_receipt = risc0_compress_proof(
                            config.stark_compression_tools_path.as_str(),
                            reciept,
                        )
                        .await
                        .map_err(|e| {
                            info!("Error compressing proof: {:?}", e);
                            Risc0RunnerError::ProofCompressionError
                        })?;

                        let (input_digest, committed_outputs) = journal.bytes.split_at(32);
                        let sig = transaction_sender
                            .submit_proof(
                                &eid,
                                claim.requester,
                                claim.program_callback.clone(),
                                &compressed_receipt.proof,
                                &compressed_receipt.execution_digest,
                                input_digest,
                                assumptions_digest.as_bytes(),
                                committed_outputs,
                                claim.additional_accounts.clone(),
                                compressed_receipt.exit_code_system,
                                compressed_receipt.exit_code_user,
                            )
                            .await
                            .map_err(|e| {
                                error!("Error submitting proof: {:?}", e);
                                Risc0RunnerError::TransactionError(e.to_string())
                            })?;

                        claim.status = ClaimStatus::Submitted;
                        claim.submission_signature = Some(sig);
                        in_flight_proofs.insert(eid.clone(), claim);
                        info!("Proof submitted: {:?}", sig);
                    }
                    Err(e) => {
                        info!("Error generating proof: {:?}", e);
                    }
                };
                in_flight_proofs.remove(&eid);
            } else {
                info!("Image not loaded, fatal error aborting execution");
            }
        }
    }
    Ok(())
}

async fn handle_execution_request<'a>(
    config: &ProverNodeConfig,
    in_flight_proofs: InflightProofRef<'a>,
    input_resolver: Arc<dyn InputResolver + 'static>,
    img_client: Arc<reqwest::Client>,
    transaction_sender: &RpcTransactionSender,
    loaded_images: LoadedImageMapRef<'a>,
    input_staging_area: InputStagingAreaRef<'a>,
    _execution_block: u64,
    exec: ExecutionRequestV1<'a>,
    accounts: &[Pubkey],
) -> Result<()> {
    if !can_execute(exec) {
        warn!(
            "Execution request for incompatible prover version: {:?}",
            exec.prover_version()
        );
        emit_event!(MetricEvents::IncompatibleProverVersion, execution_id => exec.execution_id().unwrap_or_default());
        return Ok(());
    }

    // current naive implementation is to accept everything we have pending capacity for on this node, but this needs work
    let inflight = in_flight_proofs.len();
    emit_event!(MetricEvents::ExecutionRequest, execution_id => exec.execution_id().unwrap_or_default());
    if inflight < config.maximum_concurrent_proofs as usize {
        let eid = exec
            .execution_id()
            .map(|d| d.to_string())
            .ok_or(Risc0RunnerError::InvalidData)?;
        let image_id = exec
            .image_id()
            .map(|d| d.to_string())
            .ok_or(Risc0RunnerError::InvalidData)?;
        let expiry = exec.max_block_height();
        let img = loaded_images.get(&image_id);
        let img = if img.is_none() {
            match config.missing_image_strategy {
                MissingImageStrategy::DownloadAndClaim => {
                    info!("Image not loaded, attempting to load and running claim");
                    load_image(
                        config,
                        transaction_sender,
                        &img_client,
                        &image_id,
                        loaded_images,
                    )
                    .await?;
                    loaded_images.get(&image_id)
                }
                MissingImageStrategy::DownloadAndMiss => {
                    info!("Image not loaded, loading and rejecting claim");
                    load_image(
                        config,
                        transaction_sender,
                        &img_client,
                        &image_id,
                        loaded_images,
                    )
                    .await?;
                    None
                }
                MissingImageStrategy::Fail => {
                    info!("Image not loaded, rejecting claim");
                    None
                }
            }
        } else {
            img
        }
        .ok_or(Risc0RunnerError::ImgLoadError)?;

        // naive compute cost estimate which is YES WE CAN DO THIS in the default amount of time
        emit_histogram!(MetricEvents::ImageComputeEstimate, img.size  as f64, image_id => image_id.clone());
        //ensure compute can happen before expiry
        //execution_block + (image_compute_estimate % config.max_compute_per_block) + 1 some bogus calc
        let computable_by = expiry / 2;

        if computable_by < expiry {
            //the way this is done can cause race conditions where so many request come in a short time that we accept
            // them before we change the value of g so we optimistically change to inflight and we will decrement if we dont win the claim
            let inputs = exec.input().ok_or(Risc0RunnerError::InvalidData)?;
            let program_inputs = emit_event_with_duration!(MetricEvents::InputDownload, {
                input_resolver.resolve_public_inputs(
                    inputs.iter().map(|i| i.unpack()).collect()
                ).await?
            }, execution_id => eid, stage => "public");
            input_staging_area.insert(eid.clone(), program_inputs);
            let sig = transaction_sender
                .claim(&eid, accounts[0], accounts[2], computable_by)
                .await
                .map_err(|e| Risc0RunnerError::TransactionError(e.to_string()));
            match sig {
                Ok(sig) => {
                    let callback_program = exec
                        .callback_program_id()
                        .and_then::<[u8; 32], _>(|v| v.bytes().try_into().ok())
                        .map(Pubkey::from);
                    let callback = if callback_program.is_some() {
                        Some(ProgramExec {
                            program_id: callback_program.unwrap(),
                            instruction_prefix: exec
                                .callback_instruction_prefix()
                                .map(|v| v.bytes().to_vec())
                                .unwrap_or(vec![0x1]),
                        })
                    } else {
                        None
                    };

                    in_flight_proofs.insert(
                        eid.clone(),
                        InflightProof {
                            execution_id: eid.clone(),
                            image_id: image_id.clone(),
                            status: ClaimStatus::Claiming,
                            expiry,
                            claim_signature: sig,
                            submission_signature: None,
                            requester: accounts[0],
                            program_callback: callback,
                            additional_accounts: exec
                                .callback_extra_accounts()
                                .unwrap_or_default()
                                .into_iter()
                                .map(|a| {
                                    let pkbytes: [u8; 32] = a.pubkey().into();
                                    let pubkey = Pubkey::try_from(pkbytes).unwrap_or_default();
                                    let writable = a.writable();
                                    AccountMeta {
                                        pubkey,
                                        is_writable: writable == 1,
                                        is_signer: false,
                                    }
                                })
                                .collect(),
                        },
                    );
                    emit_event!(MetricEvents::ClaimAttempt, execution_id => eid);
                }
                Err(e) => {
                    info!("Error claiming: {:?}", e);
                    in_flight_proofs.remove(&eid);
                }
            }
        }
    }
    Ok(())
}

async fn load_image<'a>(
    config: &ProverNodeConfig,
    transaction_sender: &RpcTransactionSender,
    http_client: &reqwest::Client,
    image_id: &str,
    loaded_images: LoadedImageMapRef<'a>,
) -> Result<()> {
    let account = transaction_sender
        .get_deployment_account(image_id)
        .await
        .map_err(Risc0RunnerError::ImageDownloadError)?;
    let deploy_data = root_as_deploy_v1(&account.data)
        .map_err(|_| anyhow::anyhow!("Failed to parse account data"))?;
    handle_image_deployment(config, http_client, deploy_data, loaded_images).await?;
    Ok(())
}

async fn handle_image_deployment<'a>(
    config: &ProverNodeConfig,
    http_client: &reqwest::Client,
    deploy: DeployV1<'a>,
    loaded_images: LoadedImageMapRef<'a>,
) -> Result<()> {
    let url = deploy.url().ok_or(Risc0RunnerError::InvalidData)?;
    let size = deploy.size_();
    emit_histogram!(MetricEvents::ImageDownload, size as f64, url => url.to_string());
    emit_event_with_duration!(MetricEvents::ImageDownload, {
        let resp = http_client.get(url).send().await?.error_for_status()?;
        let min = std::cmp::min(size, (config.max_image_size_mb * 1024 * 1024) as u64) as usize;
        info!("Downloading image, size {} min {}", size, min);
        if resp.status().is_success() {
            let stream = resp.bytes_stream();
            let resp_data = get_body_max_size(stream, min)
                .await
                .map_err(|_|Risc0RunnerError::ImgTooLarge)?;

            let img = Image::from_bytes(resp_data)?;
            if let Some(bytes) = img.bytes() {
                tokio::fs::write(Path::new(&config.risc0_image_folder).join(img.id.clone()), bytes).await?;
            }
            if img.id != deploy.image_id().unwrap_or_default() {
                return Err(Risc0RunnerError::InvalidData.into());
            }
            loaded_images.insert(img.id.clone(), img);
        }
        Ok(())
    }, url => url.to_string())
}

// proving function, no async this is cpu/gpu intesive
fn risc0_prove(
    memory_image: MemoryImage,
    sorted_inputs: Vec<ProgramInput>,
) -> Result<(Journal, Digest, SuccinctReceipt<ReceiptClaim>)> {
    let image_id = memory_image.compute_id().to_string();
    let mut exec = new_risc0_exec_env(memory_image, sorted_inputs)?;
    let session = exec.run()?;
    // Obtain the default prover.
    let prover = get_risc0_prover()?;
    let ctx = VerifierContext::default();
    let info = emit_event_with_duration!(MetricEvents::ProofGeneration,{
        prover.prove_session(&ctx, &session)
    }, system => "risc0")?;
    emit_histogram!(MetricEvents::ProofSegments, info.stats.segments as f64, system => "risc0", image_id => &image_id);
    emit_histogram!(MetricEvents::ProofCycles, info.stats.total_cycles as f64, system => "risc0", cycle_type => "total", image_id => &image_id);
    emit_histogram!(MetricEvents::ProofCycles, info.stats.user_cycles as f64, system => "risc0", cycle_type => "user", image_id => &image_id);
    if let InnerReceipt::Composite(cr) = &info.receipt.inner {
        let sr = emit_event_with_duration!(MetricEvents::ProofConversion,{ prover.composite_to_succinct(cr) }, system => "risc0")?;
        let ident_receipt = identity_p254(&sr)?;
        if let MaybePruned::Value(rc) = sr.claim {
            if let MaybePruned::Value(Some(op)) = rc.output {
                if let MaybePruned::Value(ass) = op.assumptions {
                    return Ok((info.receipt.journal, ass.digest(), ident_receipt));
                }
            }
        }
    }
    Err(Risc0RunnerError::ProofGenerationError.into())
}

pub struct CompressedReciept {
    pub execution_digest: Vec<u8>,
    pub exit_code_system: u32,
    pub exit_code_user: u32,
    pub proof: Vec<u8>,
}
/// Compresses the proof to be sent to the blockchain
/// This is a temporary solution until the wasm groth16 prover or a rust impl is working
async fn risc0_compress_proof(
    tools_path: &str,
    succint_receipt: SuccinctReceipt<ReceiptClaim>,
) -> Result<CompressedReciept> {
    let sealbytes = succint_receipt.get_seal_bytes();
    if !(ARCH == "x86_64" || ARCH == "x86") {
        panic!("X86 only");
    }
    let tmp = tempdir()?;
    let prove_dir = tmp.path();
    let root_path = Path::new(tools_path);
    let mut cursor = Cursor::new(&sealbytes);
    let inputs = prove_dir.join("input.json");
    let witness = prove_dir.join("out.wtns");
    let input_file = File::create(&inputs).await?;
    emit_event_with_duration!(MetricEvents::ProofConversion,{
        async_to_json(&mut cursor, input_file).await
    }, system => "groth16json")?;
    let zkey = root_path.join("stark_verify_final.zkey");
    let proof_out = prove_dir.join("proof.json");
    let public = prove_dir.join("public.json");
    emit_event_with_duration!(MetricEvents::ProofCompression,{
    let status = Command::new(root_path.join("stark_verify"))
        .arg(inputs.clone())
        .arg(witness.clone())
        .output()
        .await?;
    if !status.status.success() {
        info!("witness {:?}", status);
        return Err(Risc0RunnerError::ProofCompressionError.into());
    }
    let snark_status = Command::new(root_path.join("rapidsnark"))
        .arg(zkey)
        .arg(witness)
        .arg(proof_out.clone())
        .arg(public)
        .output()
        .await?;
    if !snark_status.status.success() {
        info!("snark {:?}", snark_status);
        return Err(Risc0RunnerError::ProofCompressionError.into());
    }
    }, system => "risc0");

    let mut proof_fd = File::open(proof_out).await?;
    let mt = proof_fd.metadata().await?;
    let mut bytes = Vec::with_capacity(mt.len() as usize);
    proof_fd.read_to_end(&mut bytes).await?;
    let proof: ProofJson = serde_json::from_slice(&bytes)?;
    let seal: Seal = proof.try_into()?;
    let claim = succint_receipt.claim;
    if let MaybePruned::Value(rc) = claim {
        let (system, user) = match rc.exit_code {
            ExitCode::Halted(user_exit) => (0, user_exit),
            ExitCode::Paused(user_exit) => (1, user_exit),
            ExitCode::SystemSplit => (2, 0),
            ExitCode::SessionLimit => (2, 2),
        };
        Ok(CompressedReciept {
            execution_digest: rc.post.digest().as_bytes().to_vec(),
            exit_code_system: system,
            exit_code_user: user,
            proof: seal.to_vec(),
        })
    } else {
        Err(Risc0RunnerError::ProofCompressionError.into())
    }
}

fn can_execute(exec: ExecutionRequestV1) -> bool {
    let version = exec.prover_version().try_into();
    if version.is_ok() {
        match version.unwrap() {
            REQUIRED_PROVER => true,
            _ => false,
        }
    } else {
        false
    }
}
