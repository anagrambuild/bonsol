use crate::input_resolver::ProgramInput;
use anyhow::Result;
use bonsol_schema::ProgramInputType;
use risc0_binfmt::MemoryImage;
use risc0_zkvm::{get_prover_server, ExecutorEnv, ExecutorImpl, ProverOpts, ProverServer, Receipt};
use std::rc::Rc;

/// Creates a new risc0 executor environment from the provided inputs, it hadles setting up the execution env in the same way across types of provers.
pub fn new_risc0_exec_env(
    image: MemoryImage,
    sorted_inputs: Vec<ProgramInput>,
) -> Result<ExecutorImpl<'static>> {
    let mut env_builder = ExecutorEnv::builder();
    for input in sorted_inputs.into_iter() {
        match input {
            ProgramInput::Resolved(ri) => {
                if ri.input_type == ProgramInputType::PublicProof {
                    let reciept: Receipt = bincode::deserialize(&ri.data)?;
                    env_builder.add_assumption(reciept);
                } else {
                    env_builder.write_slice(&ri.data);
                }
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid input type"));
            }
        }
    }
    let env = env_builder.build()?;
    ExecutorImpl::new(env, image)
}

/// Gets the default r0 prover for this application
/// Since the cli and the relay both produce proofs there is a need for a central prover configuration.
pub fn get_risc0_prover() -> Result<Rc<dyn ProverServer>> {
    let opts = ProverOpts::default();
    get_prover_server(&opts)
}
