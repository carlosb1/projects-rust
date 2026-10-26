use async_trait::async_trait;
use sqlx::{query_as, PgPool};
use uuid::Uuid;
use crate::db::repository::{CreateReport, FileRepository, Report, ReportStatus, RepositoryError};
use crate::db::types::{map_error, ReportRow, DbReportStatus};

#[derive(Clone)]
pub struct PostgresFileRepository {
    pool: PgPool,
}

impl PostgresFileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl FileRepository for PostgresFileRepository {
    async fn create(
        &self,
        input: CreateReport,
    ) -> Result<Report, RepositoryError> {
        let row = query_as::<_, ReportRow>(
            r#"
            INSERT INTO reports (report_id, path)
            VALUES ($1, $2)
            RETURNING id, report_id, path, status, created_at
            "#,
        )
            .bind(input.report_id)
            .bind(input.path)
            .fetch_one(&self.pool)
            .await
            .map_err(map_error)?;

        Ok(row.into())
    }

    async fn find_by_id(
        &self,
        id: Uuid,
    ) -> Result<Option<Report>, RepositoryError> {
        let row = query_as::<_, ReportRow>(
            r#"
            SELECT id, report_id, path, status, created_at
            FROM reports
            WHERE id = $1
            "#,
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_error)?;

        Ok(row.map(Into::into))
    }

    async fn find_by_report_id(
        &self,
        report_id: Uuid,
    ) -> Result<Option<Report>, RepositoryError> {
        let row = query_as::<_, ReportRow>(
            r#"
            SELECT id, report_id, path, status, created_at
            FROM reports
            WHERE report_id = $1
            "#,
        )
            .bind(report_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_error)?;

        Ok(row.map(Into::into))
    }

    async fn update_status(
        &self,
        report_id: Uuid,
        status: ReportStatus,
    ) -> Result<Option<Report>, RepositoryError> {
        let status = DbReportStatus::from(status);

        let row = query_as::<_, ReportRow>(
            r#"
            UPDATE reports
            SET status = $1
            WHERE report_id = $2
            RETURNING id, report_id, path, status, created_at
            "#,
        )
            .bind(status)
            .bind(report_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_error)?;

        Ok(row.map(Into::into))
    }

    async fn delete(
        &self,
        report_id: Uuid,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            "DELETE FROM reports WHERE report_id = $1",
        )
            .bind(report_id)
            .execute(&self.pool)
            .await
            .map_err(map_error)?;

        Ok(result.rows_affected() > 0)
    }

    async fn list(&self) -> Result<Vec<Report>, RepositoryError> {
        let rows = query_as::<_, ReportRow>(
            r#"
        SELECT id, report_id, path, status, created_at
        FROM reports
        ORDER BY created_at DESC, id DESC
        "#,
        )
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}