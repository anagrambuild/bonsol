use crate::{
    assertions::*,
    error::ChannelError,
    proof_handling::{
        output_digest_v1_0_1, output_digest_v1_2_1, prepare_inputs_v1_0_1, prepare_inputs_v1_2_1,
        verify_risc0_v1_0_1, verify_risc0_v1_2_1,
    },
    utilities::*,
};

use bonsol_interface::{
    bonsol_schema::{
        root_as_execution_request_v1, ChannelInstruction, ExecutionRequestV1, ExitCode, StatusV1,
    },
    prover_version::{ProverVersion, VERSION_V1_0_1, VERSION_V1_2_1},
    util::execution_address_seeds,
};

use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    instruction::{AccountMeta, Instruction},
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_memory::sol_memcmp,
    sysvar::Sysvar,
};

struct StatusAccounts<'a, 'b> {
    pub requester: &'a AccountInfo<'a>,
    pub exec: &'a AccountInfo<'a>,
    pub prover: &'a AccountInfo<'a>,
    pub callback_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub exec_bump: Option<u8>,
    pub eid: &'b str,
}

impl<'a, 'b> StatusAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b StatusV1<'b>,
    ) -> Result<Self, ChannelError> {
        let ea = &accounts[1];
        let prover = &accounts[3];
        let callback_program = &accounts[2];
        let eid = data
            .execution_id()
            .ok_or(ChannelError::InvalidExecutionAccount)?;
        let bmp = Some(check_pda(
            &execution_address_seeds(accounts[0].key, eid.as_bytes()),
            ea.key,
            ChannelError::InvalidExecutionAccount,
        )?);
        let stat = StatusAccounts {
            requester: &accounts[0],
            exec: &accounts[1],
            callback_program,
            prover,
            extra_accounts: &accounts[4..],
            exec_bump: bmp,
            eid,
        };
        Ok(stat)
    }
}

pub fn process_status_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    ix: ChannelInstruction,
) -> Result<(), ProgramError> {
    let st = ix.status_v1_nested_flatbuffer();
    if st.is_none() {
        return Err(ChannelError::InvalidInstruction.into());
    }
    let st = st.unwrap();
    let sa = StatusAccounts::from_instruction(accounts, &st)?;
    let er_ref = sa.exec.try_borrow_data()?;
    let er =
        root_as_execution_request_v1(&er_ref).map_err(|_| ChannelError::InvalidExecutionAccount)?;
    let pr_v = st.proof().filter(|x| x.len() == 256);
    if er.max_block_height() < Clock::get()?.slot {
        return Err(ChannelError::ExecutionExpired.into());
    }
    let execution_digest_v = st.execution_digest().map(|x| x.bytes());
    let input_digest_v = st.input_digest().map(|x| x.bytes());
    let assumption_digest_v = st.assumption_digest().map(|x| x.bytes());
    let committed_outputs_v = st.committed_outputs().map(|x| x.bytes());
    if let (Some(proof), Some(exed), Some(asud), Some(input_digest), Some(co)) = (
        pr_v,
        execution_digest_v,
        assumption_digest_v,
        input_digest_v,
        committed_outputs_v,
    ) {
        let proof: &[u8; 256] = proof
            .bytes()
            .try_into()
            .map_err(|_| ChannelError::InvalidInstruction)?;
        if er.verify_input_hash() {
            er.input_digest()
                .map(|x| check_bytes_match(x.bytes(), input_digest, ChannelError::InputsDontMatch));
        }
        let verified = verify_with_prover(input_digest, co, asud, er, exed, st, proof)?;
        let tip = er.tip();
        if verified {
            let callback_program_set =
                sol_memcmp(sa.callback_program.key.as_ref(), crate::ID.as_ref(), 32) != 0;
            let ix_prefix_set = er.callback_instruction_prefix().is_some();
            if callback_program_set && ix_prefix_set {
                let cbp = er
                    .callback_program_id()
                    .map(|b| b.bytes())
                    .unwrap_or(crate::ID.as_ref());
                check_bytes_match(
                    cbp,
                    sa.callback_program.key.as_ref(),
                    ChannelError::InvalidCallbackProgram,
                )?;

                let b = [sa.exec_bump.unwrap()];
                let mut seeds = execution_address_seeds(sa.requester.key, sa.eid.as_bytes());
                seeds.push(&b);
                let mut ainfos = vec![sa.exec.clone(), sa.callback_program.clone()];
                ainfos.extend(sa.extra_accounts.iter().cloned());
                // ER is the signer, it is reuired to save the execution id in the calling program
                let mut accounts = vec![AccountMeta::new_readonly(*sa.exec.key, true)];
                if let Some(extra_accounts) = er.callback_extra_accounts() {
                    if extra_accounts.len() != sa.extra_accounts.len() {
                        return Err(ChannelError::InvalidCallbackExtraAccounts.into());
                    }
                    for (i, a) in sa.extra_accounts.iter().enumerate() {
                        let stored_a = extra_accounts.get(i);
                        let key: [u8; 32] = stored_a.pubkey().into();
                        if sol_memcmp(a.key.as_ref(), &key, 32) != 0 {
                            return Err(ChannelError::InvalidCallbackExtraAccounts.into());
                        }
                        // dont cary feepayer signature through to callback we set all signer to false except the ER
                        if a.is_writable {
                            if !stored_a.writable() == 0 {
                                return Err(ChannelError::InvalidCallbackExtraAccounts.into());
                            }
                            accounts.push(AccountMeta::new(*a.key, false));
                        } else {
                            if stored_a.writable() == 1 {
                                //maybe relax this for devs?
                                return Err(ChannelError::InvalidCallbackExtraAccounts.into());
                            }
                            accounts.push(AccountMeta::new_readonly(*a.key, false));
                        }
                    }
                }
                let payload = if er.forward_output() && st.committed_outputs().is_some() {
                    [
                        er.callback_instruction_prefix().unwrap().bytes(),
                        input_digest,
                        st.committed_outputs().unwrap().bytes(),
                    ]
                    .concat()
                } else {
                    er.callback_instruction_prefix().unwrap().bytes().to_vec()
                };
                let callback_ix =
                    Instruction::new_with_bytes(*sa.callback_program.key, &payload, accounts);
                drop(er_ref);
                let res = invoke_signed(&callback_ix, &ainfos, &[&seeds]);
                match res {
                    Ok(_) => {}
                    Err(e) => {
                        msg!("{} Callback Failed: {:?}", sa.eid, e);
                    }
                }
            }
            // add curve reduction here
            payout_tip(sa.exec, sa.prover, tip)?;
            cleanup_execution_account(sa.exec, sa.requester, ExitCode::Success as u8)?;
        } else {
            msg!("{} Verifying Failed Cleaning up", sa.eid);
            cleanup_execution_account(sa.exec, sa.requester, ExitCode::VerifyError as u8)?;
        }
    } else {
        msg!("{} Proving Failed Cleaning up", sa.eid);
        cleanup_execution_account(sa.exec, sa.requester, ExitCode::ProvingError as u8)?;
    }
    Ok(())
}

fn verify_with_prover(
    input_digest: &[u8],
    co: &[u8],
    asud: &[u8],
    er: ExecutionRequestV1,
    exed: &[u8],
    st: StatusV1,
    proof: &[u8; 256],
) -> Result<bool, ProgramError> {
    let prover_version =
        ProverVersion::try_from(er.prover_version()).unwrap_or(ProverVersion::default());
    let verified = match prover_version {
        VERSION_V1_0_1 => {
            let output_digest = output_digest_v1_0_1(input_digest, co, asud);
            let proof_inputs = prepare_inputs_v1_0_1(
                er.image_id().unwrap(),
                exed,
                output_digest.as_ref(),
                st.exit_code_system(),
                st.exit_code_user(),
            )?;
            verify_risc0_v1_0_1(proof, &proof_inputs)?
        }
        VERSION_V1_2_1 => {
            let output_digest = output_digest_v1_2_1(input_digest, co, asud);
            let proof_inputs = prepare_inputs_v1_2_1(
                er.image_id().unwrap(),
                exed,
                output_digest.as_ref(),
                st.exit_code_system(),
                st.exit_code_user(),
            )?;
            verify_risc0_v1_2_1(proof, &proof_inputs)?
        }
        _ => false,
    };
    Ok(verified)
}
