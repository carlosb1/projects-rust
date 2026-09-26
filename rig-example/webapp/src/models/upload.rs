use serde::Deserialize;

#[derive(Clone, PartialEq)]
pub enum UploadStatus {
    Queued,
    Processing,
    Completed,
    Failed,
}

#[derive(Clone, PartialEq)]
pub struct UploadItem {
    pub id: String,
    pub filename: String,
    pub status: UploadStatus,
    pub progress: u8,
    pub message: Option<String>,
}

#[derive(Clone, Deserialize, PartialEq)]
pub struct UploadReport {
    pub id: String,
    pub status: String,
    pub progress: u8,
    pub message: Option<String>,
    pub processed: usize,
    pub failed: usize,
}