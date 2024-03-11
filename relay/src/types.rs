use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct BonsolInstruction {
    pub cpi: bool,
    pub accounts: Vec<Pubkey>,
    pub data: Vec<u8>,
}

pub enum CallbackStatus {
    Completed,
    Failure,
}
pub struct ProgramExec {
    pub program_id: Pubkey,
    pub instruction_prefix: Vec<u8>,
}
pub struct CallbackInstruction {
    pub execution_request_id: String,
    pub requester_account: Pubkey,
    pub execution_request_data_account: Pubkey,
    pub ix_data: Option<Vec<u8>>,
    pub program_exec: Option<ProgramExec>,
}
