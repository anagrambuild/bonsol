use solana_sdk::pubkey::Pubkey;

#[derive(Debug)]
pub struct BonsolInstruction {
    pub cpi: bool,
    pub accounts: Vec<Pubkey>,
    pub data: Vec<u8>,
    pub last_known_block: u64,
}

impl BonsolInstruction {
    pub fn new(cpi: bool, accounts: Vec<Pubkey>, data: Vec<u8>, last_known_block: u64) -> Self {
        Self {
            cpi,
            accounts,
            data,
            last_known_block,
        }
    }
}

pub enum CallbackStatus {
    Completed,
    Failure,
}
pub struct ProgramExec {
    pub program_id: Pubkey,
    pub instruction_prefix: Vec<u8>,
}
