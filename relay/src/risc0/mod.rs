use anagram_bonsol_schema::{parse_ix_data, ChannelInstructionIxType};
use anyhow::Result;
use std::{collections::HashMap, io::Read, str::from_utf8, sync::Arc};

use crate::ingest::BonsolInstruction;
use ark_bn254::{Bn254, Fr};
use ark_circom::{circom::Inputs, read_zkey, CircomBuilder, CircomConfig, WitnessCalculator};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::{Groth16, ProvingKey};

use ark_relations::r1cs::ConstraintMatrices;
use ark_std::rand::thread_rng;
use ark_std::UniformRand;
use num::BigInt;
use risc0_zkvm::{
    compute_image_id, default_prover,
    recursion::{identity_p254, join, lift},
    ExecutorEnv, VerifierContext,
};
use std::fs;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};
type GrothBn = Groth16<Bn254>;
pub struct Risc0Runner {
    loaded_images: Arc<HashMap<String, Vec<u8>>>,
    worker_handle: Option<JoinHandle<Result<()>>>,
}

fn image_id(image: &[u8]) -> String {
    hex::encode(compute_image_id(image).unwrap())
}

fn parse_image_id<'a>(raw_image_id: &'a [u8]) -> Result<&'a str> {
    let ii = from_utf8(raw_image_id)?;
    Ok(ii)
}

impl Risc0Runner {
    pub fn new(image_dir: String) -> Result<Risc0Runner> {
        let dir = fs::read_dir(image_dir)?;
        let mut loaded_images: HashMap<String, Vec<u8>> = HashMap::new();
        for entry in dir {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                let bytes = fs::read(entry.path())?;
                let image_id = hex::encode(compute_image_id(&bytes)?);
                println!("Loaded image: {}", &image_id);
                loaded_images.insert(image_id, bytes);
            }
        }

        Ok(Risc0Runner {
            loaded_images: Arc::new(loaded_images),
            worker_handle: None,
        })
    }

    pub fn start(&mut self) -> Result<UnboundedSender<BonsolInstruction>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<BonsolInstruction>();
        let loaded_images = self.loaded_images.clone();
        self.worker_handle = Some(tokio::spawn(async move {
            // let mut zkey_file = fs::File::open("stark_verify_final.zkey")?;
            // println!("Reading proving key");
            // let (pk, matrices) = read_zkey(&mut zkey_file)?;
            // println!("Proving key read");
            // let shared_pk = Arc::new(pk);
            // let shared_matrices = Arc::new(matrices);
            let mut rx = rx;

            while let Some(bix) = rx.recv().await {
                println!("Received instruction");
                // let proving_key = shared_pk.clone();
                // let matrices = shared_matrices.clone();
                let loaded_images = loaded_images.clone();
                println!("spawning task");

                let _: JoinHandle<Result<()>> = tokio::task::spawn_blocking(move || {
                    println!("Creating witness calculator");
                    let mut wtns = WitnessCalculator::new("./stark_verify.wasm")
                        .map_err(|e| anyhow::anyhow!(e))?;
                    println!("Witness calculator created");
                    if let Ok(bonsol_ix_type) = parse_ix_data(&bix.data) {
                        match bonsol_ix_type.ix_type() {
                            ChannelInstructionIxType::ExecuteV1 => {
                                if let Some(variant) = bonsol_ix_type.execute_v1_nested_flatbuffer()
                                {
                                    let image_id = parse_image_id(
                                        variant.image_id().map(|g| g.bytes()).unwrap_or(&[]),
                                    )?;
                                    let data =
                                        variant.input_data().map(|g| g.bytes()).unwrap_or(&[]);
                                    if let Some(img) = loaded_images.get(image_id) {
                                        let env = ExecutorEnv::builder()
                                            .write_slice(&data)
                                            .build()
                                            .unwrap();
                                        // Obtain the default prover.
                                        let prover = default_prover();

                                        // Produce a receipt by proving the specified ELF binary.
                                        println!("Proving image: {}", image_id);
                                        let receipt = prover.prove(env, &img).unwrap();
                                        let cr = receipt.inner.composite().unwrap();
                                        let mut rollup = lift(&cr.segments[0])?;
                                        let dctx = VerifierContext::default();
                                        for sr in &cr.segments[1..] {
                                            let rec = lift(sr)?;
                                            rollup = join(&rollup, &rec)?;
                                        }
                                        println!("Rolling up segments");
                                        // let num_inputs = matrices.num_instance_variables.clone();
                                        // let num_constraints = matrices.num_constraints.clone();
                                        
                                        // rollup.verify_integrity_with_context(&dctx)?;

                                        // let ident_receipt = identity_p254(&rollup).unwrap();
                                        // println!("Preparing Compression");

                                        // let mut inputs: HashMap<String, Inputs> = HashMap::new();

                                        // let mut rng = thread_rng();
                                        // let rng = &mut rng;
                                        // let r = ark_bn254::Fr::rand(rng);
                                        // let s = ark_bn254::Fr::rand(rng);
                                        // println!("Pushing inputs");
                                        // let biv = ident_receipt
                                        //     .seal
                                        //     .into_iter()
                                        //     .map(BigInt::from)
                                        //     .collect();
                                        // inputs.insert("iop".to_string(), biv);
                                        // let full_assignment = wtns
                                        //     .calculate_witness_element::<Bn254, _>(inputs, false)
                                        //     .unwrap();
                                        // println!("Proving Compression");
                                        // let proof =
                                        //     GrothBn::create_proof_with_reduction_and_matrices(
                                        //         &proving_key,
                                        //         r,
                                        //         s,
                                        //         &matrices,
                                        //         num_inputs,
                                        //         num_constraints,
                                        //         full_assignment.as_slice(),
                                        //     )
                                        //     .unwrap();
                                        // println!("Proof: {:?}", proof);
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
