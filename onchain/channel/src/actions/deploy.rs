use crate::assertions::*;
use crate::error::ChannelError;
use crate::utilities::*;
use bonsol_channel_utils::deployment_address_seeds;
use bonsol_channel_utils::img_id_hash;
use bonsol_schema::ChannelInstruction;
use bonsol_schema::DeployV1;
use solana_program::account_info::AccountInfo;
use solana_program::msg;
use solana_program::system_program;

pub struct DeployAccounts<'a, 'b> {
    pub deployer: &'a AccountInfo<'a>,
    pub payer: &'a AccountInfo<'a>,
    pub deployment: &'a AccountInfo<'a>,
    pub system_program: &'a AccountInfo<'a>,
    pub extra_accounts: &'a [AccountInfo<'a>],
    pub deployment_bump: Option<u8>,
    pub image_id: &'b str,
    pub image_checksum: &'b [u8],
}

impl<'a, 'b> DeployAccounts<'a, 'b> {
    fn from_instruction(
        accounts: &'a [AccountInfo<'a>],
        data: &'b DeployV1<'b>,
    ) -> Result<Self, ChannelError> {
        if let Some(imageid) = data.image_id() {
            let mut da = DeployAccounts {
                deployer: &accounts[0],
                payer: &accounts[1],
                deployment: &accounts[2],
                system_program: &accounts[3],
                extra_accounts: &accounts[4..],
                deployment_bump: None,
                image_id: imageid,
                image_checksum: &[],
            };
            let owner = data
                .owner()
                .map(|b| b.bytes())
                .ok_or(ChannelError::InvalidInstruction)?;
            check_writable_signer(da.payer, ChannelError::InvalidPayerAccount)?;
            check_writable_signer(da.deployer, ChannelError::InvalidDeployerAccount)?;
            check_bytes_match(
                da.deployer.key.as_ref(),
                owner,
                ChannelError::InvalidDeployerAccount,
            )?;
            check_writeable(da.deployment, ChannelError::InvalidDeploymentAccount)?;
            check_owner(
                da.deployment,
                &system_program::ID,
                ChannelError::InvalidDeploymentAccount,
            )?;
            ensure_0(da.deployment, ChannelError::InvalidDeploymentAccount)?;
            check_key_match(
                da.system_program,
                &system_program::ID,
                ChannelError::InvalidInstruction,
            )?;

            da.deployment_bump = Some(check_pda(
                &deployment_address_seeds(&img_id_hash(imageid)),
                da.deployment.key,
                ChannelError::InvalidDeploymentAccountPDA,
            )?);
            return Ok(da);
        }

        Err(ChannelError::InvalidInstruction)
    }
}

pub fn process_deploy_v1<'a>(
    accounts: &'a [AccountInfo<'a>],
    ix: ChannelInstruction<'a>,
) -> Result<(), ChannelError> {
    msg!("deploy");
    let dp = ix.deploy_v1_nested_flatbuffer();
    if dp.is_none() {
        return Err(ChannelError::InvalidInstruction.into());
    }
    let dp = dp.unwrap();
    let da = DeployAccounts::from_instruction(accounts, &dp)?;
    let b = [da.deployment_bump.unwrap()];
    let imghash = img_id_hash(da.image_id);

    let mut seeds = deployment_address_seeds(&imghash);
    seeds.push(&b);
    let dp_bytes = ix.deploy_v1().unwrap().bytes();
    save_structure(
        da.deployment,
        &seeds,
        dp_bytes,
        da.payer,
        da.system_program,
        None,
    )
}
