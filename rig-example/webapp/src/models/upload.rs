use std::fmt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Clone, PartialEq)]
pub enum UploadStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Clone, PartialEq)]
pub struct UploadItem {
    pub id: Option<Uuid>,
    pub filename: String,
    pub status: UploadStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

impl fmt::Display for ReportStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            ReportStatus::Pending => "pending",
            ReportStatus::Processing => "processing",
            ReportStatus::Completed => "completed",
            ReportStatus::Failed => "failed",
        };

        write!(f, "{value}")
    }
}



#[derive(Clone, Deserialize, Serialize, PartialEq)]
pub struct Report {
    pub id: Uuid,
    pub report_id: Uuid,
    pub path: String,
    pub status: ReportStatus,
    pub created_at: DateTime<Utc>,
}
