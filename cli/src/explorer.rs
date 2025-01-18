use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, Result as SqliteResult};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum RequestStatus {
    Pending,
    InProgress { prover_id: String },
    Completed { result: String },
    Failed { error: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub id: String,
    pub status: RequestStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub request_data: String, // The original execution request data
}

pub struct Explorer {
    conn: Connection,
}

impl Explorer {
    pub fn new() -> Result<Self, anyhow::Error> {
        let conn = Connection::open("bonsol_explorer.db")?;
        
        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS execution_requests (
                id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                request_data TEXT NOT NULL,
                prover_id TEXT,
                result TEXT,
                error TEXT
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    pub fn track_request(&mut self, request_data: String) -> Result<String, anyhow::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        self.conn.execute(
            "INSERT INTO execution_requests (id, status, created_at, updated_at, request_data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                &id,
                "Pending",
                now.to_rfc3339(),
                now.to_rfc3339(),
                &request_data,
            ),
        )?;

        Ok(id)
    }

    pub fn get_request_status(&self, request_id: &str) -> Result<RequestStatus, anyhow::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT status, prover_id, result, error FROM execution_requests WHERE id = ?1"
        )?;
        
        let (status, prover_id, result, error) = stmt.query_row([request_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        })?;

        Ok(match status.as_str() {
            "Pending" => RequestStatus::Pending,
            "InProgress" => RequestStatus::InProgress { 
                prover_id: prover_id.unwrap_or_default() 
            },
            "Completed" => RequestStatus::Completed { 
                result: result.unwrap_or_default() 
            },
            "Failed" => RequestStatus::Failed { 
                error: error.unwrap_or_default() 
            },
            _ => return Err(anyhow::anyhow!("Invalid status in database")),
        })
    }

    pub fn list_requests(&self) -> Result<Vec<ExecutionRequest>, anyhow::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, status, created_at, updated_at, request_data, prover_id, result, error 
             FROM execution_requests 
             ORDER BY created_at DESC"
        )?;

        let requests = stmt.query_map([], |row| {
            let status = match row.get::<_, String>(1)?.as_str() {
                "Pending" => RequestStatus::Pending,
                "InProgress" => RequestStatus::InProgress { 
                    prover_id: row.get::<_, Option<String>>(5)?.unwrap_or_default() 
                },
                "Completed" => RequestStatus::Completed { 
                    result: row.get::<_, Option<String>>(6)?.unwrap_or_default() 
                },
                "Failed" => RequestStatus::Failed { 
                    error: row.get::<_, Option<String>>(7)?.unwrap_or_default() 
                },
                _ => return Err(rusqlite::Error::InvalidParameterCount(1, 1)),
            };

            Ok(ExecutionRequest {
                id: row.get(0)?,
                status,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?).unwrap().into(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?).unwrap().into(),
                request_data: row.get(4)?,
            })
        })?;

        Ok(requests.collect::<SqliteResult<Vec<_>>>()?)
    }

    pub fn update_request_status(
        &self,
        request_id: &str,
        new_status: RequestStatus,
    ) -> Result<(), anyhow::Error> {
        let now = Utc::now();
        
        match new_status {
            RequestStatus::Pending => {
                self.conn.execute(
                    "UPDATE execution_requests 
                     SET status = 'Pending', updated_at = ?1 
                     WHERE id = ?2",
                    (now.to_rfc3339(), request_id),
                )?;
            }
            RequestStatus::InProgress { prover_id } => {
                self.conn.execute(
                    "UPDATE execution_requests 
                     SET status = 'InProgress', prover_id = ?1, updated_at = ?2 
                     WHERE id = ?3",
                    (prover_id, now.to_rfc3339(), request_id),
                )?;
            }
            RequestStatus::Completed { result } => {
                self.conn.execute(
                    "UPDATE execution_requests 
                     SET status = 'Completed', result = ?1, updated_at = ?2 
                     WHERE id = ?3",
                    (result, now.to_rfc3339(), request_id),
                )?;
            }
            RequestStatus::Failed { error } => {
                self.conn.execute(
                    "UPDATE execution_requests 
                     SET status = 'Failed', error = ?1, updated_at = ?2 
                     WHERE id = ?3",
                    (error, now.to_rfc3339(), request_id),
                )?;
            }
        }

        Ok(())
    }

    pub fn assign_to_prover(&self, request_id: &str, prover_id: &str) -> Result<(), anyhow::Error> {
        let now = Utc::now();
        
        self.conn.execute(
            "UPDATE execution_requests 
             SET status = 'InProgress', 
                 prover_id = ?1, 
                 updated_at = ?2 
             WHERE id = ?3 AND status = 'Pending'",
            (prover_id, now.to_rfc3339(), request_id),
        )?;

        Ok(())
    }

    pub fn mark_completed(&self, request_id: &str, result: &str) -> Result<(), anyhow::Error> {
        let now = Utc::now();
        
        self.conn.execute(
            "UPDATE execution_requests 
             SET status = 'Completed', 
                 result = ?1, 
                 updated_at = ?2 
             WHERE id = ?3 AND status = 'InProgress'",
            (result, now.to_rfc3339(), request_id),
        )?;

        Ok(())
    }

    pub fn mark_failed(&self, request_id: &str, error: &str) -> Result<(), anyhow::Error> {
        let now = Utc::now();
        
        self.conn.execute(
            "UPDATE execution_requests 
             SET status = 'Failed', 
                 error = ?1, 
                 updated_at = ?2 
             WHERE id = ?3",
            (error, now.to_rfc3339(), request_id),
        )?;

        Ok(())
    }

    pub fn get_pending_requests(&self) -> Result<Vec<ExecutionRequest>, anyhow::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, status, created_at, updated_at, request_data 
             FROM execution_requests 
             WHERE status = 'Pending'
             ORDER BY created_at ASC"
        )?;

        let requests = stmt.query_map([], |row| {
            Ok(ExecutionRequest {
                id: row.get(0)?,
                status: RequestStatus::Pending,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?).unwrap().into(),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?).unwrap().into(),
                request_data: row.get(4)?,
            })
        })?;

        Ok(requests.collect::<SqliteResult<Vec<_>>>()?)
    }

    pub fn get_prover_requests(&self, _prover_id: &str) -> Result<Vec<ExecutionRequest>, anyhow::Error> {
        let _stmt = self.conn.prepare(
            "SELECT id, status, created_at, updated_at, request_data, result, error 
             FROM execution_requests 
             WHERE prover_id = ?1
             ORDER BY updated_at DESC"
        )?;

        // ... similar to list_requests implementation ...
        todo!()
    }
} 
