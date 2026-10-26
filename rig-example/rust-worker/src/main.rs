mod ai_agent;
pub mod db;
pub mod jobs;
mod storage;

use std::path::PathBuf;
use std::sync::Arc;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_sqs::Client;
use sqlx::postgres::PgPoolOptions;
use tokio::fs;
use uuid::Uuid;
use crate::db::pg_repository::PostgresFileRepository;
use crate::db::repository::{FileRepository, Report, RepositoryError};
use crate::jobs::RemoteJob;
use crate::storage::{Storage, AWS_REGIOM, StorageError};

const DIR_UPLOADS: &str = "./uploads";

pub async fn initialize_dir_uploads() -> anyhow::Result<()> {
    fs::create_dir_all(&DIR_UPLOADS)
        .await.map_err(|e| anyhow::anyhow!("Failed to create uploads directory: {}", e))?;
    Ok(())
}
pub async fn delete_directory_uploads(path_buf: &PathBuf)  -> anyhow::Result<()> {
    fs::remove_dir_all(path_buf).await.map_err(|e| {
        tracing::error!("Failed to delete uploads directory: {}", e);
        anyhow::anyhow!("Failed to delete uploads directory: {}", e)
    })?;
    Ok(())
}

pub async fn delete_files_in_uploads(path_buf: &PathBuf) -> anyhow::Result<()> {
    match fs::metadata(path_buf).await {
        Ok(metadata) => {
            if metadata.is_dir() {
                fs::remove_dir_all(path_buf).await?;
            } else {
                fs::remove_file(path_buf).await?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!("Upload path not found, skipping delete: {}", path_buf.display());
        }
        Err(e) => {
            tracing::error!("Failed to inspect upload path {}: {}", path_buf.display(), e);
            return Err(anyhow::anyhow!("Failed to delete uploads directory: {}", e));
        }
    }
    Ok(())
}



pub async fn get_upload_path_file(job_id: &Uuid) -> Result<PathBuf, RepositoryError> {
    Ok(std::path::Path::new(&format!("{}/{}", DIR_UPLOADS, job_id)).to_path_buf())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    tracing::info!("Starting server on port 3000");


    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::warn!("DATABASE_URL not set, using default postgres::5432/mydb");
        "postgres://rigadmin:postgres@localhost:5432/rig".to_string()
    });

    // Initialize SQS
    let endpoint = std::env::var("SQS_ENDPOINT")?;

    // Setting up s3
    let s3_endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| {
        tracing::warn!("S3_ENDPOINT not set, using default http://localhost:9000");
        "http://localhost:9000".to_string()
    });

    let s3_bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| {
        tracing::warn!("S3_BUCKET not set, using default 'my-bucket'");
        "my-bucket".to_string()
    });

    let credentials = Credentials::new(
        std::env::var("RUSTFS_ACCESS_KEY")?,
        std::env::var("RUSTFS_SECRET_KEY")?,
        None,
        None,
        "rustfs",
    );

    let qdrant_url = std::env::var("QDRANT_URL")
        .unwrap_or_else(|_| "http://localhost:6334".to_owned());

    tracing::info!("Using variables");
    tracing::info!("DATABASE_URL: {}", database_url);
    tracing::info!("S3_ENDPOINT: {}", s3_endpoint);
    tracing::info!("S3_BUCKET: {}", s3_bucket);
    tracing::info!("QDRANT_URL: {}", qdrant_url);
    tracing::info!("RUSTFS_ACCESS_KEY: {}", std::env::var("RUSTFS_ACCESS_KEY").unwrap_or_else(|_| "not set".to_string()));
    tracing::info!("--------------------------------");

    // Setting postgres
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let db = Arc::new(PostgresFileRepository::new(pool));



    // Storage
    tracing::info!("Checking uploads directory");
    initialize_dir_uploads().await?;

    let storage = match Storage::new(DIR_UPLOADS.to_string(), AWS_REGIOM, s3_endpoint.as_str(), s3_bucket.as_str(), credentials).await  {
        Ok(storage) => storage,
        Err(StorageError::CreateError) => {
            tracing::error!("Failed to create storage");
            return Err("Failed to create storage".into());
        }
        Err(StorageError::PermissionDenied) => {
            tracing::error!("Permission denied");
            return Err("Permission denied".into());
        }
    };



    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new("eu-west-1"))
        .endpoint_url(&endpoint)
        .load()
        .await;

    let client = Client::new(&config);

    let queue_url = std::env::var("QUEUE_URL")?;

    tracing::info!("Waiting for messages... for queue: {queue_url}");

    loop {
        let response = client
            .receive_message()
            .queue_url(&queue_url)
            .max_number_of_messages(10)
            .wait_time_seconds(20)
            .send()
            .await;

        match response {
            Ok(response) => {
                for message in response.messages() {
                    if let Some(body) = message.body() {
                        tracing::info!("Received: {body}");
                        println!("Received: {body}");

                        let payload: RemoteJob = match serde_json::from_str(body) {
                            Ok(v) => v,
                            Err(err) => {
                                tracing::warn!("JSON invalid: {body}. Error: {err}");
                                continue;
                            }
                        };

                        tracing::info!("Processing job: {:?}", payload);


                        let destination_path = get_upload_path_file(&payload.job_id).await?;
                        let report_key = format!("report_{}.zip", payload.job_id);
                        storage.download_from_s3(
                            &report_key.to_string(),
                            &destination_path
                        ).await?;

                        tracing::info!("Download report id");
                        let report = match db.find_by_report_id(payload.job_id).await {
                            Ok(Some(report)) => {report}
                            Ok(None) => {
                                tracing::warn!("Report not found: {}", payload.job_id);
                                continue;
                            }
                            Err(err) => {
                                tracing::error!("Failed to find report {}: {}", payload.job_id, err);
                                continue;
                            }
                        };


                        process_message(&report).await;
                        delete_files_in_uploads(&destination_path).await?;

                        //let report = db.find_by_report_id();
                        // Aquí procesas el mensaje
                        // process_message(body).await;

                        if let Some(receipt_handle) = message.receipt_handle() {
                            client
                                .delete_message()
                                .queue_url(&queue_url)
                                .receipt_handle(receipt_handle)
                                .send()
                                .await?;
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!("SQS error: {error}");
                tokio::time::sleep(
                    std::time::Duration::from_secs(5)
                ).await;
            }
        }
    }
}

pub async fn process_message(report: &Report) {
    tracing::info!("Report found: {:?}", report);
}