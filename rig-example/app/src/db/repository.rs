use async_trait::async_trait;
use sqlx::Type;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize)]
#[sqlx(type_name = "report_status", rename_all = "snake_case")]
pub enum ReportStatus {
    Pending,
    Indexing,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Report {
    pub id: Uuid,
    pub report_id: Uuid,
    pub path: String,
    pub status: ReportStatus,
    pub created_at: DateTime<Utc>,
}

pub struct CreateReport {
    pub report_id: Uuid,
    pub path: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Report already exists")]
    AlreadyExists,

    #[error("Storage error: {0}")]
    Storage(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn create(
        &self,
        input: CreateReport,
    ) -> Result<Report, RepositoryError>;

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Report>, RepositoryError>;

    async fn find_by_report_id(
        &self,
        report_id: Uuid,
    ) -> Result<Option<Report>, RepositoryError>;

    async fn update_status(
        &self,
        report_id: Uuid,
        status: ReportStatus,
    ) -> Result<Option<Report>, RepositoryError>;

    async fn delete(
        &self,
        report_id: Uuid,
    ) -> Result<bool, RepositoryError>;
}
