use {
    crate::input_resolver::ProgramInput,
    anyhow::Result,
    bonsol_schema::ProgramInputType,
    risc0_binfmt::MemoryImage,
    risc0_zkvm::{
        get_prover_server, ExecutorEnv, ExecutorImpl, ProverOpts, ProverServer, Receipt,
    }, std::rc::Rc,
};

pub fn new_risc0_exec_env(image: MemoryImage, sorted_inputs: Vec<ProgramInput>) -> Result<ExecutorImpl<'static>> {
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

pub fn get_risc0_prover() -> Result<Rc<dyn ProverServer>> {
  let opts = ProverOpts::default();
  get_prover_server(&opts)
}


// pub fn risc0_prove(
//     memory_image: MemoryImage,
//     sorted_inputs: Vec<ProgramInput>,
// ) -> Result<(Journal, Digest, SuccinctReceipt<ReceiptClaim>)> {
//     let mut exec = ExecutorImpl::new(env, memory_image)?;

//     l

//     // Obtain the default prover.
    
//     let prover = get_prover_server(&opts)?;
//     let info = emit_event_with_duration!(MetricEvents::ProofGeneration,{
//       prover.prove_session(&ctx, &session)
//   }, system => "risc0")?;
//     emit_histogram!(MetricEvents::ProofSegments, info.stats.segments as f64, system => "risc0");
//     emit_histogram!(MetricEvents::ProofCycles, info.stats.total_cycles as f64, system => "risc0", cycle_type => "total");
//     emit_histogram!(MetricEvents::ProofCycles, info.stats.user_cycles as f64, system => "risc0", cycle_type => "user");
//     if let InnerReceipt::Composite(cr) = &info.receipt.inner {
//         let sr = emit_event_with_duration!(MetricEvents::ProofConversion,{ prover.composite_to_succinct(&cr)}, system => "risc0")?;
//         let ident_receipt = identity_p254(&sr)?;
//         if let MaybePruned::Value(rc) = sr.claim {
//             if let MaybePruned::Value(Some(op)) = rc.output {
//                 if let MaybePruned::Value(ass) = op.assumptions {
//                     return Ok((info.receipt.journal, ass.digest(), ident_receipt));
//                 }
//             }
//         }
//     }
//     return Err(Risc0RunnerError::ProofGenerationError.into());
// }
