use anagram_bonsol_schema::{
    parse_ix_data, ChannelInstruction, ChannelInstructionArgs, ChannelInstructionIxType,
     ExecutionInputType, StatusTypes, StatusV1, StatusV1Args,
};
use anyhow::Result;
use flatbuffers::FlatBufferBuilder;
use hex::FromHex;
use risc0_groth16::{
    docker::stark_to_snark, split_digest, to_json, verifier::prepared_verifying_key, Seal, Verifier,
};
use solana_sdk::pubkey::Pubkey;
use std::convert::TryInto;

use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use std::{collections::HashMap, fs::File, io::Cursor, str::from_utf8, sync::Arc};
use wasmer::{Engine, Module, Store};

use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G2Affine};
use ark_circom::{
    circom::Inputs, read_zkey_mapped, CircomBuilder, CircomConfig, WitnessCalculator,
};
use ark_crypto_primitives::snark::SNARK;
use ark_groth16::Groth16;
use ark_serialize::CanonicalSerialize;

use ark_relations::r1cs::ConstraintMatrices;
use ark_std::rand::thread_rng;
use ark_std::UniformRand;
use num::BigInt;
use risc0_zkvm::{
    compute_image_id, default_prover, get_prover_server,
    recursion::{identity_p254, join, lift, valid_control_ids},
    sha::{Digest, Digestible},
    CompactReceipt, ExecutorEnv, ExecutorImpl, ProverOpts, VerifierContext, ALLOWED_IDS_ROOT,
};
use std::fs;
use tokio::{sync::mpsc::UnboundedSender, task::JoinHandle};

use crate::types::{BonsolInstruction, CallbackInstruction, ProgramExec};
type GrothBn = Groth16<Bn254>;
pub struct Risc0Runner {
    loaded_images: Arc<HashMap<String, Vec<u8>>>,
    worker_handle: Option<JoinHandle<Result<()>>>,
    callback_channel: UnboundedSender<CallbackInstruction>,
}

fn image_id(image: &[u8]) -> String {
    hex::encode(compute_image_id(image).unwrap())
}

fn parse_image_id<'a>(raw_image_id: &'a [u8]) -> Result<String> {
    let ii = from_utf8(raw_image_id)?;
    Ok(ii.to_string())
}

impl Risc0Runner {
    pub fn new(
        image_dir: String,
        callback_channel: UnboundedSender<CallbackInstruction>,
    ) -> Result<Risc0Runner> {
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
            callback_channel,
        })
    }

    pub fn start(&mut self) -> Result<UnboundedSender<BonsolInstruction>> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<BonsolInstruction>();
        let loaded_images = self.loaded_images.clone();
        let mut rx = rx;
        let callback_channel = self.callback_channel.clone();
        self.worker_handle = Some(tokio::spawn(async move {
            while let Some(bix) = rx.recv().await {
                println!("Received instruction");

                let loaded_images = loaded_images.clone();

                let bonsol_ix_type = parse_ix_data(&bix.data)?;

                let variant = match bonsol_ix_type.ix_type() {
                    ChannelInstructionIxType::ExecuteV1 => bonsol_ix_type
                        .execute_v1_nested_flatbuffer()
                        .ok_or(anyhow::anyhow!("Invalid execute instruction")),
                    _ => Err(anyhow::anyhow!("Invalid instruction type")),
                }?;

                let image_id =
                    parse_image_id(variant.image_id().map(|g| g.bytes()).unwrap_or(&[]))?;
                //TODO need to sandbox this somehow. could be malicious. Downloading any of this unchecked data could be fundamentally broken
                let data_type = variant.input_type();

                let data = match data_type {
                    ExecutionInputType::DATA => variant
                        .input_data()
                        .map(|g| g.bytes().to_vec())
                        .unwrap_or(Vec::new()),
                    ExecutionInputType::URL => {
                        // could brick us
                        let url =
                            from_utf8(variant.input_data().map(|g| g.bytes()).unwrap_or(&[]))?;
                        // async http client with retrys and timeouts
                        // ensure size of payload is factored into tip settings

                        Vec::new()
                    }
                    _ => Vec::new(),
                };
                let callback_ix_prefix = variant
                    .callback_instruction_prefix()
                    .map(|g| g.bytes().to_vec());
                let execution_id = variant.execution_id().map(|g| g.bytes().to_vec()).unwrap();

                let handle: JoinHandle<Result<CallbackInstruction>> =
                    tokio::task::spawn_blocking(move || {
                        if let Some(img) = loaded_images.get(&image_id) {
                            let env = ExecutorEnv::builder().write_slice(&data).build()?;
                            let mut exec = ExecutorImpl::from_elf(env, &img).unwrap();
                            let session = exec.run().unwrap();
                            // Obtain the default prover.
                            let opts = ProverOpts::default();
                            let ctx = VerifierContext::default();
                            let prover = get_prover_server(&opts).unwrap();
                            let receipt = prover.prove_session(&ctx, &session).unwrap();
                            let claim: risc0_zkvm::ReceiptClaim = receipt.get_claim().unwrap();
                            let composite_receipt = receipt.inner.composite().unwrap();
                            let succinct_receipt = prover.compress(composite_receipt).unwrap();
                            let ident_receipt = identity_p254(&succinct_receipt).unwrap();
                            let s = ident_receipt.get_seal_bytes().clone();
                            println!("Proving Compression");
                            let seal = stark_to_snark(&s)?;
                            let (a0, a1) = split_digest(Digest::from_hex(ALLOWED_IDS_ROOT)?)?;
                            let (c0, c1) = split_digest(claim.digest())?;
                            let input = &[a0, a1, c0, c1];
                            let mut input_vec = Vec::new();
                            let sealbytes: [u8; 256] = seal
                                .to_vec()
                                .try_into()
                                .map_err(|_| anyhow::anyhow!("Seal is the wrong size"))?;
                            input.serialize_compressed(&mut input_vec)?;
                            let inputbytes: Vec<u8> = input_vec
                                .try_into()
                                .map_err(|_| anyhow::anyhow!("Input is the wrong size"))?;
                            let mut fbb = FlatBufferBuilder::new();
                            let s = fbb.create_vector(&sealbytes);
                            let i = fbb.create_vector(&inputbytes);
                            let stat = StatusV1::create(
                                &mut fbb,
                                &StatusV1Args {
                                    status: StatusTypes::Completed,
                                    proof: Some(s),
                                    input: Some(i),
                                },
                            );
                            fbb.finish(stat, None);
                            let statbytes = fbb.finished_data();
                            let mut fbb2 = FlatBufferBuilder::new();
                            let off = fbb2.create_vector(statbytes);
                            let root = ChannelInstruction::create(
                                &mut fbb2,
                                &ChannelInstructionArgs {
                                    ix_type: ChannelInstructionIxType::StatusV1,
                                    execute_v1: None,
                                    status_v1: Some(off),
                                },
                            );
                            fbb2.finish(root, None);
                            let buf = fbb2.finished_data();
                            let requester = bix.accounts[0];
                            let er = bix.accounts[1];
                            let cb_program = bix.accounts[2];
                            let cb = CallbackInstruction {
                                execution_request_id: execution_id,
                                requester_account: requester,
                                execution_request_data_account: er,
                                ix_data: Some(buf.to_vec()),
                                program_exec: if callback_ix_prefix.is_some() {
                                    Some(ProgramExec {
                                        program_id: cb_program,
                                        instruction_prefix: callback_ix_prefix.unwrap(),
                                    })
                                } else {
                                    None
                                },
                            };
                            return Ok(cb);
                            //TODO allow extra accounts function via interface
                            // TODO figure out why all rust/wasm rpoofs not working so close 
                            // let mut rng = thread_rng();
                            // println!("Preparing Compression");
                            // let mut inputs: HashMap<String, Inputs> = HashMap::new();
                            // println!("Pushing inputs");
                            // let biv = ident_receipt.seal.into_iter().map(BigInt::from).collect();
                            // inputs.insert("iop".to_string(), Inputs::BigIntVec(biv));
                            // let store = Store::default();
                            // let mut wtns =
                            //     WitnessCalculator::from_module(&module, store).map_err(|e| {
                            //         println!("{:?}", e);
                            //         anyhow::anyhow!(e)
                            //     })?;
                            // let full_assignment = wtns
                            //     .calculate_witness_element::<Bn254, _>(inputs, false)
                            //     .unwrap();
                            //    match comp.verify_integrity() {
                            //         Ok(t) => {
                            //             println!("Verification: {:?}", t);
                            //         }
                            //         Err(e) => {
                            //             println!("Verification failed: {:?}", e);
                            //         }
                            //    }

                            //     let rng = &mut rng;
                            //     let r = ark_bn254::Fr::rand(rng);
                            //     let s = ark_bn254::Fr::rand(rng);
                            //     let proof =
                            //         GrothBn::create_proof_with_reduction_and_matrices(
                            //             &proving_key,
                            //             r,
                            //             s,
                            //             &matrices,
                            //             num_inputs,
                            //             num_constraints,
                            //             full_assignment.as_slice(),
                            //         )
                            //         .unwrap();
                            //     println!("Proof: {:?}", proof);
                            //     use hex::FromHex;
                            //     let (a0, a1) =
                            //         split_digest(Digest::from_hex(ALLOWED_IDS_ROOT)?)?;
                            //     let (c0, c1) = split_digest(claim.digest())?;
                            //     let vk = File::open("verification_key.json")?;
                            //     let vk: risc0_groth16::VerifyingKeyJson =
                            //         serde_json::from_reader(vk)?;
                            //     let pvk = vk.prepared_verifying_key()?;

                            //     let inputs = &[
                            //         a0, a1, c0, c1
                            //     ];
                            //     let valid =
                            //         GrothBn::verify_with_processed_vk(&pvk, inputs, &proof);

                            //     match valid {
                            //         Ok(t) => {
                            //             println!("Verification: {:?}", t);
                            //         }
                            //         Err(e) => {
                            //             println!("Verification failed: {:?}", e);
                            //         }
                            //     }
                        }
                        Err(anyhow::anyhow!("Image not found"))
                    });
                match handle.await {
                    Ok(Ok(cb)) => {
                        if let Err(e) = callback_channel.send(cb) {
                            println!("Callback channel failed: {:?}", e);
                        }
                    }
                    Ok(Err(e)) => {
                        println!("Error: {:?}", e);
                    }
                    Err(e) => {
                        println!("Error: {:?}", e);
                    }
                }
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
