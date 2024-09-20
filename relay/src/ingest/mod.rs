mod dragon;
mod rpc;
use anyhow::Result;
pub use {dragon::GrpcIngester, rpc::RpcIngester};

use {
    solana_sdk::pubkey::Pubkey,
    std::{
        error::Error,
        fmt::{self, Display, Formatter},
    },
    tokio::sync::mpsc::UnboundedReceiver,
};

use crate::types::BonsolInstruction;
pub type TxChannel = UnboundedReceiver<Vec<BonsolInstruction>>;
#[derive(Debug)]
pub enum IngestErrorType {
    RpcError,
    IoError,
}
#[derive(Debug)]
pub struct IngestError {
    pub code: IngestErrorType,
    pub message: String,
}
impl Display for IngestError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "IngestError: {:?} - {:?}", self.code, self.message)
    }
}
impl Error for IngestError {}

pub type IngesterResult = Result<(), IngestError>;
pub trait Ingester {
    fn start(&mut self, program: Pubkey) -> Result<TxChannel>;

    fn stop(&mut self) -> Result<()>;
}
