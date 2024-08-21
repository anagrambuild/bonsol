use crate::assertions::*;
use crate::error::ChannelError;
use crate::utilities::*;
use anagram_bonsol_channel_utils::execution_address_seeds;
use anagram_bonsol_schema::root_as_deploy_v1;
use anagram_bonsol_schema::root_as_input_set;
use anagram_bonsol_schema::ChannelInstruction;
use anagram_bonsol_schema::ExecutionRequestV1;
use anagram_bonsol_schema::InputType;
use solana_program::account_info::AccountInfo;
use solana_program::system_program;
use solana_program::bpf_loader_upgradeable;

pub struct ExecuteAccounts<'a, 'b> {
    pub requester: &'a AccountInfo<'a>,
    pub payer: &'a AccountInfo<'a>,
    pub exec: &'a AccountInfo<'a>,
    pub deployment: &'a AccountInfo<'a>,
    pub callback_program: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub execution_id: &'b str,
    pub exec_bump: Option<u8>,
}

impl<'a, 'b> ExecuteAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b ExecutionRequestV1<'b>,
    ) -> Result<Self, ChannelError> {
        if let Some(executionid) = data.execution_id() {
            let evec = executionid;
            let mut ea = ExecuteAccounts {
                requester: &accounts[0],
                payer: &accounts[1],
                exec: &accounts[2],
                deployment: &accounts[3],
                callback_program: &accounts[4],
                system_program: &accounts[5],
                extra_accounts: &accounts[6..],
                execution_id: evec,
                exec_bump: None,
            };
            check_writable_signer(ea.requester, ChannelError::InvalidRequesterAccount)?;
            check_writable_signer(ea.payer, ChannelError::InvalidPayerAccount)?;
            check_writeable(ea.exec, ChannelError::InvalidExecutionAccount)?;
            check_owner(
                ea.exec,
                &system_program::ID,
                ChannelError::InvalidExecutionAccount,
            )?;
            ensure_0(ea.exec, ChannelError::InvalidExecutionAccount)?;
            check_owner(
                ea.deployment,
                &crate::ID,
                ChannelError::InvalidDeploymentAccount,
            )?;
            let deploy_data = &*ea
                .deployment
                .try_borrow_data()
                .map_err(|_| ChannelError::InvalidDeploymentAccount)?;
            let deploy = root_as_deploy_v1(&*&deploy_data)
                .map_err(|_| ChannelError::InvalidDeploymentAccount)?;

            let inputs = data.input().ok_or(ChannelError::InvalidInputs)?;
            // this should never be less than 1
            let required_input_size = deploy.inputs().map(|x| x.len()).unwrap_or(1);
            let mut num_sets = 0;
            let input_set: usize = inputs
                .iter()
                .filter(|i| {
                    // these must be changed on client to reference account index, the will be 1 byte
                    i.data().is_some() && i.input_type() == InputType::InputSet
                })
                .flat_map(|i| {
                    num_sets += 1;
                    // can panic here
                    let index = i.data().map(|x| x.bytes().get(0)).flatten().unwrap();
                    let rel_index = index - 6;
                    let account = ea
                        .extra_accounts
                        .get(rel_index as usize)
                        .ok_or(ChannelError::InvalidInputs)
                        .unwrap();
                    let data = account.data.borrow();
                    let input_set =
                        root_as_input_set(&*data).map_err(|_| ChannelError::InvalidInputs)?;
                    input_set
                        .inputs()
                        .map(|x| x.len())
                        .ok_or(ChannelError::InvalidInputs)
                })
                .fold(0, |acc, x| acc + x);

            if inputs.len() - num_sets + input_set != required_input_size {
                return Err(ChannelError::InvalidInputs);
            }
            //todo ensure inputs ar correct types/public private on chain, provers do this onsite so its low priority
            ea.exec_bump = Some(check_pda(
                &execution_address_seeds(&ea.requester.key, evec.as_bytes()),
                ea.exec.key,
                ChannelError::InvalidExecutionAccount,
            )?);

            if data.max_block_height() == 0 {
                return Err(ChannelError::MaxBlockHeightRequired);
            }

            if data.verify_input_hash() && data.input_digest().is_none() {
                return Err(ChannelError::InputDigestRequired);
            }

            or(
                &[
                    check_key_match(
                        ea.callback_program,
                        &crate::ID,
                        ChannelError::InvalidCallbackAccount,
                    ),
                    check_owner(
                        ea.callback_program,
                        &bpf_loader_upgradeable::ID,
                        ChannelError::InvalidCallbackAccount,
                    ),
                ],
                ChannelError::InvalidCallbackAccount,
            )?;
            check_key_match(
                ea.system_program,
                &system_program::ID,
                ChannelError::InvalidInstruction,
            )?;
            return Ok(ea);
        }

        Err(ChannelError::InvalidInstruction)
    }
}
//todo forward extra accounts
pub fn process_execute_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    ix: ChannelInstruction<'a>,
) -> Result<(), ChannelError> {
    let er = ix.execute_v1_nested_flatbuffer();
    if er.is_none() {
        return Err(ChannelError::InvalidInstruction.into());
    }
    let er = er.unwrap();
    let ea = ExecuteAccounts::from_instruction(accounts, &er)?;
    let b = [ea.exec_bump.unwrap()];
    let mut seeds = execution_address_seeds(ea.requester.key, ea.execution_id.as_bytes());
    seeds.push(&b);
    let bytes = ix.execute_v1().unwrap().bytes();
    save_structure(ea.exec, &seeds, bytes, ea.payer, ea.system_program, None)
}
