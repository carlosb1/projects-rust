pub mod ai_agent;

use std::sync::Arc;
use anyhow::Result;
use axum::{Json, Router};
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use rig::serde::Deserialize;
use tokio::sync::{broadcast, Mutex};
use tokio::sync::broadcast::Sender;
use tower_http::services::{ServeDir, ServeFile};

struct AppState {
    tx: Sender<String>,
    //db_client: Arc<Mutex<db::Client>>,
}

use dashmap::DashMap;
use tokio::sync::mpsc;

type QueueId = String;

struct Job {
    payload: String,
}

struct JobQueues {
    queues: DashMap<QueueId, mpsc::Sender<Job>>,
}

impl JobQueues {
    fn new() -> Self {
        Self {
            queues: DashMap::new(),
        }
    }

    fn create_queue(
        &self,
        id: QueueId,
    ) -> mpsc::Receiver<Job> {
        let (tx, rx) = mpsc::channel(100);

        self.queues.insert(id, tx);

        rx
    }

    async fn push(
        &self,
        id: &str,
        job: Job,
    ) -> Result<(), &'static str> {
        let tx = self
            .queues
            .get(id)
            .ok_or("queue not found")?
            .clone();

        tx.send(job)
            .await
            .map_err(|_| "queue closed")
    }
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

use axum::extract::Multipart;

async fn upload(
    State(arc_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<StatusCode, StatusCode> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let filename = field
            .file_name()
            .map(ToString::to_string);

        let data = field
            .bytes()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        tracing::info!(
            "Received {:?}: {} bytes",
            filename,
            data.len()
        );

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


#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting server on port 3000");

    let queues = Arc::new(JobQueues::new());

    let mut rx = queues.create_queue("model-provisioning".into());

    tokio::spawn(async move {
        while let Some(job) = rx.recv().await {
            println!("processing {}", job.payload);
        }
    });

    let api_routes = Router::new()
        .route("/health", get(health_check))
        .route("/search", get(search))
        .route("/reports", post(post_reports));

    // build our application with a single route
    let app = Router::new().nest("/api", api_routes)
        .fallback_service(
            ServeDir::new("dist")
                .not_found_service(ServeFile::new("dist/index.html"))
        )
        .with_state(Arc::new(AppState {
            tx: broadcast::channel(100).0,
            //db_client: Arc::new(Mutex::new(db::Client::new(qdrant_host.as_str(), qdrant_port, None)?)),
        }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())

}

// which calls one of these handlers
async fn root() {}

async fn post_reports() {}

