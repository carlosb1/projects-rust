pub mod ai_agent;
pub mod storage;
mod queue;
mod jobs;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::Result;
use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::http::{header, Method, StatusCode};
use axum::routing::{get, post};
use rig::serde::Deserialize;
use tokio::sync::{broadcast, Mutex};
use tokio::sync::broadcast::Sender;
use tower_http::services::{ServeDir, ServeFile};
use axum::extract::Multipart;
use uuid::{Bytes, Uuid};

use dashmap::DashMap;
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use crate::jobs::{Job, JobPublisher};
use crate::queue::{Queue, QueueJob, JobStatus};
use crate::storage::Storage;


struct AppState {
    queue: Arc<Queue>,
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
    mut multipart: Multipart,
) -> Result<StatusCode, StatusCode> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let storage = Storage::new(DIR_UPLOADS.to_string());
        let (id, path) = storage.new_file_path_with_uuid()
            .await?;

        // Download a new file, when it is finished
        storage.download_file(field, &path)
            .await?;

        // enqueue for a batch process
        state.queue.enqueue(path, id)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // guardar ZIP / procesarlo / crear job...
    }

    Ok(StatusCode::ACCEPTED)
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

pub struct ReportService {

}

impl ReportService {
    pub fn analyze_directory(&self, path: &Path) {

    }
}

async fn process_report(job: &QueueJob, report_service: Arc<ReportService>, job_publisher: Arc<Option<JobPublisher>>) -> anyhow::Result<()> {
    let data = tokio::fs::read(&job.file_path).await?;

    report_service.analyze_directory(&job.file_path);
    tracing::info!(
        "Analyzing report {}: {} bytes",
        job.id,
        data.len()
    );
    let  remote_job = Job {
        job_id: job.id,
        task: "analyze_report".to_string(),
        document_key: job.file_path.to_string_lossy().to_string(),
    };
    if let Some(job_publisher) = job_publisher.as_ref() {
        job_publisher.publish_python(&remote_job.clone()).await?;
        job_publisher.publish_rust(&remote_job.clone()).await?;
    } else {
        tracing::warn!("No job publisher configured, skipping publishing job {}", job.id);
    }


    // TODO:
    // 1. Validar ZIP
    // 2. Extraer documentos
    // 3. Analizar cada documento
    // 4. Guardar resultados

    Ok(())
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting server on port 3000");

    let (tx, rx) = mpsc::channel::<QueueJob>(100);
    let queue = Arc::new(Queue::new(tx));
    let worker_jobs = queue.jobs.clone();

    let report_service = Arc::new(ReportService{});

    let publisher = if let Ok(job_publisher) = JobPublisher::from_env().await {
        Arc::new(Some(job_publisher))
    } else {
        Arc::new(None)
    };

    tokio::spawn(async move {
        run_worker(rx, worker_jobs, report_service.clone(), publisher.clone() ).await;
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
        .route("/reports", post(post_reports))
        .layer(cors);

    // build our application with a single route
    let app = Router::new().nest("/api", api_routes)
        .fallback_service(
            ServeDir::new("dist")
                .not_found_service(ServeFile::new("dist/index.html"))
        )
        .with_state(Arc::new(AppState {
            queue: queue.clone(),
            //db_client: Arc::new(Mutex::new(db::Client::new(qdrant_host.as_str(), qdrant_port, None)?)),
        }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(ADDR).await.expect("Failed to bind to address");
    axum::serve(listener, app).await.expect("Failed to run server");
    Ok(())
}

