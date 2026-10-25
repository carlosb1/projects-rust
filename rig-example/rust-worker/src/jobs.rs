use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteJob {
    pub job_id: Uuid,
    pub task: String,
    pub document_key: String,
}
