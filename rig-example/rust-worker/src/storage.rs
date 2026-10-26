
use std::path::Path;
use anyhow::anyhow;
use aws_sdk_s3::Client;
use aws_sdk_s3::config::{BehaviorVersion, Credentials, Region};
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::{config::Builder};
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
    
    pub async fn new(dir_uploads: String, aws_region: &str, endpoint: &str, bucket: &str, credentials: Credentials) -> Result<Self, StorageError> {
        let s3_client = Self::new_s3_client(aws_region, endpoint, credentials).await;
        Ok(Storage { dir_uploads, s3_client, name_bucket: bucket.to_string() })
    }

    fn new_id() -> Uuid {
        Uuid::new_v4()
    }

    pub async fn download_from_s3(
        &self,
        key: &str,
        destination: &Path,
    ) -> anyhow::Result<()> {
        let response = self.s3_client
            .get_object()
            .bucket(&self.name_bucket)
            .key(key)
            .send()
            .await
            .map_err(|err| {
                tracing::error!("Error downloading S3 object {key}: {err}");

                if err.as_service_error()
                    .is_some_and(|error| error.is_no_such_key())
                {
                   anyhow!("Error downloading S3 object: {err}")
                } else {
                    anyhow!("Error downloading S3 object: {err}")
                }
            })?;

        // Crear el directorio de destino si es necesario.
        if let Some(parent) = destination.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .await?;
            }
        }

        // File::create sobrescribe el fichero si ya existe.
        let mut file = fs::File::create(destination)
            .await?;

        let mut reader = response.body.into_async_read();

        tokio::io::copy(&mut reader, &mut file)
            .await?;

        Ok(())
    }



}
