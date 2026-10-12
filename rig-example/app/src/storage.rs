
use std::path::Path;
use axum::extract::multipart::Field;
use axum::http::StatusCode;
use tokio::fs;
use uuid::Uuid;

pub struct Storage {
    dir_uploads: String,

}

impl Storage {
    pub fn new(dir_uploads: String) -> Self {
        Storage { dir_uploads }
    }

    fn new_id() -> Uuid {
        Uuid::new_v4()
    }

    pub async fn new_file_path_with_uuid(&self) -> Result<(Uuid,std::path::PathBuf), StatusCode> {
        // Create directory
        fs::create_dir_all(&self.dir_uploads)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Initialize id
        let new_uuid = Uuid::new_v4();

        // Initialize path
        let path = Path::new(&self.dir_uploads).join(format!("report_{}.zip", new_uuid));
        let uuid = Self::new_id();
        Ok((uuid, path))
    }

    pub async fn download_file(&self, field: Field<'_>, path: &Path) -> Result<std::path::PathBuf, StatusCode> {
        let filename = field
            .file_name()
            .map(ToString::to_string);
        // Download data
        let data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        fs::write(&path, &data)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        tracing::info!(
            "Received {:?}: {} bytes",
            path.file_name(),
            data.len()
        );
        Ok(path.to_path_buf())
    }
}
