use tokio::sync::mpsc::UnboundedSender;
mod rpc;
mod dragon;
pub use dragon::GrpcIngester;
pub use rpc::RpcIngester;
use {
    anyhow::Result,
    futures_util::StreamExt,
    solana_pubsub_client::nonblocking::pubsub_client::PubsubClient,
    solana_rpc_client_api::config::{RpcBlockSubscribeConfig, RpcBlockSubscribeFilter},
    solana_sdk::{bs58, commitment_config::CommitmentConfig},
};

use {
    solana_sdk::pubkey::Pubkey,
    solana_transaction_status::{
        EncodedTransactionWithStatusMeta, UiInnerInstructions, UiInstruction, UiTransactionEncoding,
    },
    std::{
        error::Error,
        fmt::{self, Display, Formatter},
    },
    tokio::{sync::mpsc::UnboundedReceiver, task::JoinHandle},
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
