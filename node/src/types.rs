use {
    solana_sdk::{instruction::CompiledInstruction, message::AccountKeys, pubkey::Pubkey},
    solana_transaction_status::InnerInstruction,
};

#[derive(Debug)]
pub struct BonsolInstruction {
    /// Whether we picked up an instruction that was an inner instruction
    /// found in the metadata.
    pub cpi: bool,
    pub accounts: Vec<Pubkey>,
    pub data: Vec<u8>,
    pub last_known_block: u64,
}

impl BonsolInstruction {
    pub const fn new(
        cpi: bool,
        accounts: Vec<Pubkey>,
        data: Vec<u8>,
        last_known_block: u64,
    ) -> Self {
        Self {
            cpi,
            accounts,
            data,
            last_known_block,
        }
    }
    const fn inner(accounts: Vec<Pubkey>, data: Vec<u8>, last_known_block: u64) -> Self {
        Self::new(true, accounts, data, last_known_block)
    }
    const fn outer(accounts: Vec<Pubkey>, data: Vec<u8>, last_known_block: u64) -> Self {
        Self::new(false, accounts, data, last_known_block)
    }
}

/// Conversion trait for Inner and Outer instructions to become Bonsol instructions.
/// This does not use From and Into because it requires context other than just
/// the instructions themselves.
pub trait IntoBonsolInstruction {
    /// Convert an instruction into a [`BonsolInstruction`].
    fn into_bonsol_ix(self, acc: &AccountKeys, last_known_block: u64) -> BonsolInstruction;
    /// Get the index of the `Pubkey` that represents the program in the `AccountKeys` map.
    fn program_id_index(&self) -> u8;
}

impl IntoBonsolInstruction for InnerInstruction {
    fn into_bonsol_ix(self, acc: &AccountKeys, last_known_block: u64) -> BonsolInstruction {
        BonsolInstruction::inner(
            self.instruction
                .accounts
                .into_iter()
                .filter_map(|idx| acc.get(idx as usize).copied())
                .collect(),
            self.instruction.data,
            last_known_block,
        )
    }
    fn program_id_index(&self) -> u8 {
        self.instruction.program_id_index
    }
}

impl IntoBonsolInstruction for CompiledInstruction {
    fn into_bonsol_ix(self, acc: &AccountKeys, last_known_block: u64) -> BonsolInstruction {
        BonsolInstruction::outer(
            self.accounts
                .into_iter()
                .filter_map(|idx| acc.get(idx as usize).copied())
                .collect(),
            self.data,
            last_known_block,
        )
    }
    fn program_id_index(&self) -> u8 {
        self.program_id_index
    }
}

/// Filter instructions that can be converted to `BonsolInstruction`s given
/// a closure which returns a boolean representing some condition that must be met
/// for an instruction to be converted to `Some(BonsolInstruction)`.
pub fn filter_bonsol_instructions<'a, I>(
    ixs: Vec<I>,
    acc: &'a AccountKeys,
    program: &'a Pubkey,
    last_known_block: u64,
    program_filter: impl Fn(&AccountKeys, &Pubkey, usize) -> bool + 'a,
) -> impl Iterator<Item = BonsolInstruction> + 'a
where
    I: IntoBonsolInstruction + 'a,
{
    ixs.into_iter().filter_map(move |ix| {
        program_filter(acc, program, ix.program_id_index() as usize)
            .then(|| ix.into_bonsol_ix(acc, last_known_block))
    })
}

pub enum CallbackStatus {
    Completed,
    Failure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramExec {
    pub program_id: Pubkey,
    pub instruction_prefix: Vec<u8>,
}
