
pub mod storage;
mod queue;
mod remote_jobs;
pub mod db;

use anyhow::Result;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::routing::{get, post};
use axum::{Json, Router};
use std::path::Path;
use std::sync::Arc;
use aws_sdk_s3::config::Credentials;
use tower_http::services::{ServeDir, ServeFile};
use uuid::Uuid;

use crate::queue::{JobStatus, Queue, QueueJob};
use crate::remote_jobs::{JobPublisher, RemoteJob};
use crate::storage::{AWS_REGIOM, Storage, StorageError};
use dashmap::DashMap;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use crate::db::pg_repository::PostgresFileRepository;
use crate::db::repository::{FileRepository, Report};
use crate::db::search_engine;

struct AppState {
    queue: Arc<Queue>,
    storage: Arc<Storage>,
    report_service: Arc<ReportService>,
    search_service: Arc<SearchService>
    //db_client: Arc<Mutex<db::Client>>,
}


//TODO add endpoints for agents
async fn search(State(arc_state): State<Arc<AppState>>,
                Query(mut params):  Query<SearchQuery>) -> Json<SearchResponse> {
    tracing::info!("search params: {:?}", params);
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(10);

    // llamar Qdrant / DB aquí...

    Json(SearchResponse {
        items: vec![],
        page,
        total_pages: 1,
        total: 0,
    })
}


#[derive(Deserialize, Debug)]
struct SearchQuery {
    q: String,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SearchResult {
    result: String,
}



#[derive(serde::Serialize, serde::Deserialize)]
pub struct SearchResponse {
    pub items: Vec<SearchResult>,
    pub page: usize,
    pub total_pages: usize,
    pub total: usize,
}



#[derive(serde::Serialize)]
    struct HealthStatus {
    status: String,
}


const DIR_UPLOADS: &str = "./uploads";



async fn post_reports(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Uuid>, StatusCode> {
    tracing::info!("Posting a new report");
    let filename = headers
        .get("x-filename")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("report.zip");
        tracing::info!("Uploading ZIP: {} bytes, filename={}", body.len(), filename);

        let storage = state.storage.clone();
        let (id, path) = storage.new_file_path_with_uuid()
            .await?;

        // Download a new file, when it is finished
        tracing::info!("Download file locally");
        tokio::fs::write(&path, &body)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        tracing::info!("File downloaded to {:?} and sending to the queue as id={:?}", path, id);
        // enqueue for a batch process
        state.queue.enqueue(path, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        tracing::info!("Sent file");

    Ok(Json(id))
}

#[axum_macros::debug_handler]
async fn get_report(
    axum::extract::Path(id): axum::extract::Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Report>, StatusCode> {

    //TODO checK TO rename file id or job id as glossary
    let report = state.report_service.db.find_by_report_id(id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match report {
        Some(report) => Ok(Json(report)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn health_check() -> Result<Json<HealthStatus>, StatusCode> {
    tracing::info!("health check");
    let data = HealthStatus {
        status: "OK".to_string(),
    };
    Ok(Json(data))
}

const ADDR: &'static str = "0.0.0.0:3000";

pub async fn run_worker(
    mut rx: mpsc::Receiver<QueueJob>,
    jobs: Arc<DashMap<Uuid, JobStatus>>,
    report_service: Arc<ReportService>,
    job_publisher: Arc<Option<JobPublisher>>
) {
    while let Some(job) = rx.recv().await {
        tracing::info!("Processing job {}", job.id);

        jobs.insert(job.id, JobStatus::Processing);

        let result = process_report(&job,report_service.clone(), job_publisher.clone()).await;

        match result {
            Ok(()) => {
                jobs.insert(job.id, JobStatus::Completed);
                tracing::info!("Job {} completed", job.id);
            }
            Err(err) => {
                tracing::error!("Job {} failed: {}", job.id, err);

                jobs.insert(
                    job.id,
                    JobStatus::Failed(err.to_string()),
                );
            }
        }
    }
}

pub struct SearchService {
    pub db: search_engine::Client
}

impl SearchService {
    pub async fn search(&mut self, text: Vec<String>) -> anyhow::Result<Vec<crate::db::search_engine::SearchResult>>{
        tracing::info!("Searching");
        self.db.search(text).await
    }
}

pub struct ReportService {
    pub storage: Arc<Storage>,
    pub db: Arc<PostgresFileRepository>,
}

impl ReportService {
    pub fn new(storage: Arc<Storage>, db: Arc<PostgresFileRepository>) -> Self {
        Self {
            storage,
            db
        }
    }
    pub async fn analyze_directory(&self, path: &Path, id: Uuid) {
        self.storage.upload_to_s3(&format!("report_{}.zip", id), path)
            .await
            .expect("Failed to upload report to S3");

    }
}

async fn process_report(job: &QueueJob, report_service: Arc<ReportService>, job_publisher: Arc<Option<JobPublisher>>) -> anyhow::Result<()> {
    let data = tokio::fs::read(&job.file_path).await?;

    report_service.db.create(crate::db::repository::CreateReport {
        report_id: job.id,
        path: job.file_path.to_string_lossy().to_string(),
    }).await?;

    report_service.analyze_directory(&job.file_path, job.id).await;
    tracing::info!(
        "Uploading and Analyzing report{}: {} bytes",
        job.id,
        data.len()
    );

    // Do we want to keep the status here..
    report_service.db.update_status(job.id, crate::db::types::ReportStatus::Indexing).await?;

    let remote_job = RemoteJob {
        job_id: job.id,
        task: "analyze_report".to_string(),
        document_key:  "mocked_key_not_local_file_path".to_string(),
    };
    if let Some(job_publisher) = job_publisher.as_ref() {
        job_publisher.publish_python(&remote_job.clone()).await?;
        job_publisher.publish_rust(&remote_job.clone()).await?;
    } else {
        tracing::warn!("No job publisher configured, skipping publishing job {}", job.id);
    }

    Ok(())
}

fn print_variables() {
    for (key, value) in std::env::vars() {
        tracing::info!("{key}: {value}");
    }
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();
    print_variables();
    tracing::info!("Starting server on port 3000");


    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        tracing::warn!("DATABASE_URL not set, using default postgres::5432/mydb");
        "postgres://rigadmin:postgres@localhost:5432/rig".to_string()
    });

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

    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or("http://0.0.0.0:6333".to_string());

    tracing::info!("Using variables");
    tracing::info!("DATABASE_URL: {}", database_url);
    tracing::info!("S3_ENDPOINT: {}", s3_endpoint);
    tracing::info!("S3_BUCKET: {}", s3_bucket);
    tracing::info!("QDRANT_URL: {}", qdrant_url);
    tracing::info!("S3_cREDENTIALS: {:?}", credentials);
    tracing::info!("--------------------------------");

    // Setting postgres
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    //  Running migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    // Initialize db
    let db = Arc::new(PostgresFileRepository::new(pool));

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
    let storage = Arc::new(storage);


    let (tx, rx) = mpsc::channel::<QueueJob>(100);
    let queue = Arc::new(Queue::new(tx));
    let worker_jobs = queue.jobs.clone();


    let client  = search_engine::Client::new(qdrant_url, None)?;
    let search_service = Arc::new(SearchService {
        db: client
    });

    let report_service = Arc::new(ReportService{ storage: storage.clone(), db: db.clone() });

    let publisher = if let Ok(job_publisher) = JobPublisher::from_env().await {
        tracing::info!("Loading job publisher");
        Arc::new(Some(job_publisher))
    } else {
        tracing::warn!("Failed to load job publisher");
        Arc::new(None)
    };

   let cloned_service = report_service.clone();
    tokio::spawn(async move {
        run_worker(rx, worker_jobs, cloned_service, publisher.clone() ).await;
    });

    //It is not necessary for webapp local
    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:8000".parse::<axum::http::HeaderValue>().expect("Invalid header value for cors")
        )
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/search", get(search))
        .route("/uploads", post(post_reports))
        .route("/uploads/{id}/report", get(get_report))
        .layer(cors);




    // build our application with a single route
    let app = Router::new().nest("/api", api_routes)
        .fallback_service(
            ServeDir::new("dist")
                .not_found_service(ServeFile::new("dist/index.html"))
        )
        .with_state(Arc::new(AppState {
            queue: queue.clone(),
            storage: storage.clone(),
            report_service: report_service.clone(),
            search_service: search_service.clone()
            //db_client: Arc::new(Mutex::new(db::Client::new(qdrant_host.as_str(), qdrant_port, None)?)),
        }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(ADDR).await.expect("Failed to bind to address");
    axum::serve(listener, app).await.expect("Failed to run server");
    Ok(())
}

