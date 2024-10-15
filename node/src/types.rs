use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct BonsolInstruction {
    pub cpi: bool,
    pub accounts: Vec<Pubkey>,
    pub data: Vec<u8>,
    pub last_known_block: u64,
}

pub enum CallbackStatus {
    Completed,
    Failure,
}
pub struct ProgramExec {
    pub program_id: Pubkey,
    pub instruction_prefix: Vec<u8>,
}
