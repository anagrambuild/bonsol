use crate::assertions::*;
use crate::error::ChannelError;
use crate::utilities::*;
use bonsol_channel_interface::{
    bonsol_channel_utils::input_set_address_seeds,
    bonsol_schema::input_set_op_v1_generated::InputSetOp,
    bonsol_schema::input_set_op_v1_generated::InputSetOpV1,
    bonsol_schema::ChannelInstruction,
};
use solana_program::account_info::AccountInfo;
use solana_program::system_program;

pub struct InputSetAccounts<'a, 'b> {
    pub payer: &'a AccountInfo<'a>,
    pub input_set: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub input_set_bump: Option<u8>,
    pub input_set_id: &'b str,
}

impl<'a, 'b> InputSetAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b InputSetOpV1<'b>,
    ) -> Result<Self, ChannelError> {
        let id = data.id().ok_or(ChannelError::InvalidInputSetData)?;

        let mut ia = InputSetAccounts {
            payer: &accounts[0],
            input_set: &accounts[1],
            system_program: &accounts[2],
            extra_accounts: &accounts[3..],
            input_set_bump: None,
            input_set_id: id,
        };
        check_writable_signer(ia.payer, ChannelError::InvalidPayerAccount)?;
        check_writeable(ia.input_set, ChannelError::InvalidInputSetAccount)?;
        check_owner(
            ia.input_set,
            &system_program::ID,
            ChannelError::InvalidInputSetAccount,
        )?;
        if data.op() == InputSetOp::Create {
            ensure_0(ia.input_set, ChannelError::InvalidInputSetAccount)?;
        }
        if data.op() != InputSetOp::Delete && data.inputs().is_none() {
            return Err(ChannelError::InvalidInstruction.into());
        }
        ia.input_set_bump = Some(check_pda(
            &input_set_address_seeds(id.as_bytes()),
            ia.input_set.key,
            ChannelError::InvalidInputSetAccount,
        )?);
        Ok(ia)
    }
}

pub fn process_input_set_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    ix: ChannelInstruction<'a>,
) -> Result<(), ChannelError> {
    let is = ix.input_set_v1_nested_flatbuffer();
    if is.is_none() {
        return Err(ChannelError::InvalidInstruction.into());
    }
    let is = is.unwrap();
    let sa = InputSetAccounts::from_instruction(accounts, &is)?;
    let mut seeds = input_set_address_seeds(sa.input_set_id.as_bytes());
    let bump = [sa.input_set_bump.unwrap()];
    seeds.push(&bump);
    let bytes = ix.input_set_v1().unwrap().bytes();
    save_structure(
        sa.input_set,
        &seeds,
        bytes,
        sa.payer,
        sa.system_program,
        None,
    )
}
