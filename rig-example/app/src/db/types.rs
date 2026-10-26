use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;
pub(crate) use crate::db::repository::{Report, ReportStatus, RepositoryError};


#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "report_status", rename_all = "snake_case")]
pub enum DbReportStatus {
    Pending,
    Indexing,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, FromRow)]
pub struct ReportRow {
    pub id: Uuid,
    pub report_id: Uuid,
    pub path: String,
    pub status: DbReportStatus,
    pub created_at: DateTime<Utc>,
}

impl From<DbReportStatus> for ReportStatus {
    fn from(value: DbReportStatus) -> Self {
        match value {
            DbReportStatus::Pending => Self::Pending,
            DbReportStatus::Processing => Self::Processing,
            DbReportStatus::Completed => Self::Completed,
            DbReportStatus::Failed => Self::Failed,
            DbReportStatus::Indexing => Self::Indexing,
        }
    }
}

impl From<ReportStatus> for DbReportStatus {
    fn from(value: ReportStatus) -> Self {
        match value {
            ReportStatus::Pending => Self::Pending,
            ReportStatus::Processing => Self::Processing,
            ReportStatus::Completed => Self::Completed,
            ReportStatus::Failed => Self::Failed,
            ReportStatus::Indexing => Self::Indexing,
        }
    }
}

impl From<ReportRow> for Report {
    fn from(row: ReportRow) -> Self {
        Self {
            id: row.id,
            report_id: row.report_id,
            path: row.path,
            status: row.status.into(),
            created_at: row.created_at,
        }
    }
}

pub fn map_error(error: sqlx::Error) -> RepositoryError {
    if let sqlx::Error::Database(db_error) = &error {
        if db_error.is_unique_violation() {
            return RepositoryError::AlreadyExists;
        }
    }

    RepositoryError::Storage(Box::new(error))
}