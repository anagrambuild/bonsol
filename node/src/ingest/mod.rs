mod dragon;
mod rpc;

use anyhow::Result;
pub use {dragon::GrpcIngester, rpc::RpcIngester};

use {
    crate::types::BonsolInstruction, solana_sdk::pubkey::Pubkey,
    tokio::sync::mpsc::UnboundedReceiver,
};

pub type TxChannel = UnboundedReceiver<Vec<BonsolInstruction>>;

#[derive(Debug, thiserror::Error)]
pub enum IngestErrorType {
    #[error("RPC Error")]
    RpcError,
    #[error("I/O Error")]
    IoError,
}

#[derive(Debug, thiserror::Error)]
#[error("IngestError: {code} - {message}")]
pub struct IngestError {
    pub code: IngestErrorType,
    pub message: String,
}

pub type IngesterResult = Result<(), IngestError>;
pub trait Ingester {
    fn start(&mut self, program: Pubkey) -> Result<TxChannel>;

    fn stop(&mut self) -> Result<()>;
}

#[cfg(test)]
mod test {

    #[test]
    fn test_ingest_error_type_display() {
        let rpc_error = super::IngestErrorType::RpcError;
        let io_error = super::IngestErrorType::IoError;

        assert_eq!(rpc_error.to_string(), "RPC Error");
        assert_eq!(io_error.to_string(), "I/O Error");
    }

    #[test]
    fn test_ingest_error_display() {
        let rpc_error = super::IngestError {
            code: super::IngestErrorType::RpcError,
            message: "RPC failed".to_string(),
        };

        let io_error = super::IngestError {
            code: super::IngestErrorType::IoError,
            message: "I/O failed".to_string(),
        };

        assert_eq!(rpc_error.to_string(), "IngestError: RPC Error - RPC failed");
        assert_eq!(io_error.to_string(), "IngestError: I/O Error - I/O failed");
    }
}
