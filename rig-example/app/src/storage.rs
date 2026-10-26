
use std::path::Path;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::{config::Builder};
use axum::extract::multipart::Field;
use axum::http::StatusCode;
use tokio::fs;
use uuid::Uuid;

pub struct Storage{
    dir_uploads: String,
    s3_client: Client,
    name_bucket: String,
}
pub const AWS_REGIOM : &str = "eu-west-1";

pub enum StorageError {
    PermissionDenied,
    CreateError,
}

impl Storage {
    async fn new_s3_client(aws_region: &str, endpoint: &str, credentials: Credentials) -> Client {
        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new(aws_region.to_owned()))
            .endpoint_url(endpoint)
            .credentials_provider(credentials)
            .load()
            .await;

        let s3_config = Builder::from(&shared_config)
            .force_path_style(true)
            .build();
        Client::from_conf(s3_config)
    }

     async fn create_bucket_if_not_exists(client: &Client, name_bucket: &str) -> Result<(), StorageError> {
         match client.head_bucket().bucket(name_bucket).send().await {
             Ok(_) => {
                 tracing::info!("The bucket already exists: {}", name_bucket);
                 Ok(())
             }
             Err(err) => {
                 tracing::error!("Error creating bucket: {:?}", err);
                 Self::check_error(&err).ok_or(StorageError::PermissionDenied)?;
                 let _ = client.create_bucket().bucket(name_bucket).send().await.map_err(|_| StorageError::CreateError)?;
                 Ok(())
             }
         }
    }
    fn check_error<E>(err: &SdkError<E>) -> bool {
        if let SdkError::ServiceError(service_err) = err {
            service_err.raw().status().as_u16() == 404
        } else {
            false
        }
    }
    pub async fn new(dir_uploads: String, aws_region: &str, endpoint: &str, bucket: &str, credentials: Credentials) -> Result<Self, StorageError> {
        let s3_client = Self::new_s3_client(aws_region, endpoint, credentials).await;
        let _ = Self::create_bucket_if_not_exists(&s3_client, bucket).await?;
        Ok(Storage { dir_uploads, s3_client, name_bucket: bucket.to_string() })
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

    pub async fn upload_to_s3(&self, key: &str, file_path: &Path) -> Result<(), StatusCode> {
        let body = aws_sdk_s3::primitives::ByteStream::from_path(file_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        self.s3_client.put_object()
            .bucket(self.name_bucket.to_owned())
            .key(key)
            .body(body)
            .send()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub async fn reset(&self) -> Result<(), StatusCode> {
        fs::remove_dir_all(&self.dir_uploads)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }


}
