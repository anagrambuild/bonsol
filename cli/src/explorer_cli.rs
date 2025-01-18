use anyhow::Result;
use clap::Subcommand;
use crate::explorer::{Explorer, RequestStatus};

#[derive(Subcommand)]
pub enum ExplorerCommand {
    /// List all execution requests
    List,
    /// Get status of a specific request
    Status {
        #[clap(name = "request-id")]
        request_id: String,
    },
    /// Create a new test request
    Create {
        /// Test data for the request
        #[clap(name = "data")]
        data: String,
    },
    /// Update status of a request
    Update {
        #[clap(name = "request-id")]
        request_id: String,
        /// New status (pending, in_progress, completed, failed)
        #[clap(name = "status")]
        status: String,
        /// Additional data (prover_id for in_progress, result for completed, error for failed)
        #[clap(name = "data")]
        data: Option<String>,
    },
    /// List pending requests
    ListPending,
    
    /// List requests for a specific prover
    ListProver {
        #[clap(name = "prover-id")]
        prover_id: String,
    },
}

pub fn handle_explorer_command(command: ExplorerCommand) -> Result<()> {
    let mut explorer = Explorer::new()?;

    Ok(match command {
        ExplorerCommand::List => {
            let requests = explorer.list_requests()?;
            if requests.is_empty() {
                println!("No execution requests found.");
                return Ok(());
            }

            for request in requests {
                println!("Request ID: {}", request.id);
                println!("Status: {:?}", request.status);
                println!("Created: {}", request.created_at);
                println!("Updated: {}", request.updated_at);
                println!("---");
            }
            ()
        },
        ExplorerCommand::Status { request_id } => {
            let status = explorer.get_request_status(&request_id)?;
            println!("Status for request {}: {:?}", request_id, status);
            ()
        },
        ExplorerCommand::Create { data } => {
            let request_id = explorer.track_request(data)?;
            println!("Created new request with ID: {}", request_id);
            ()
        },
        ExplorerCommand::Update { request_id, status, data } => {
            let new_status = match status.as_str() {
                "pending" => RequestStatus::Pending,
                "in_progress" => RequestStatus::InProgress { 
                    prover_id: data.unwrap_or_default() 
                },
                "completed" => RequestStatus::Completed { 
                    result: data.unwrap_or_default() 
                },
                "failed" => RequestStatus::Failed { 
                    error: data.unwrap_or_default() 
                },
                _ => return Err(anyhow::anyhow!("Invalid status. Use: pending, in_progress, completed, or failed")),
            };

            explorer.update_request_status(&request_id, new_status)?;
            println!("Request status updated successfully!");
            ()
        },
        ExplorerCommand::ListPending => {
            let requests = explorer.get_pending_requests()?;
            println!("Pending requests: {:?}", requests);
            ()
        },
        ExplorerCommand::ListProver { prover_id } => {
            let requests = explorer.get_prover_requests(&prover_id)?;
            println!("Prover requests: {:?}", requests);
            ()
        }
    })
} 
