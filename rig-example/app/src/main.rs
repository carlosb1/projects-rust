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

struct AppState {
    tx: Sender<String>,
    //db_client: Arc<Mutex<db::Client>>,
}

#[derive(Debug, Deserialize)]
struct Q {
    q: Vec<String>,
}



#[derive(serde::Serialize)]
struct SearchResult {
    result: String,
}

//TODO add endpoints for agents
async fn search(State(arc_state): State<Arc<AppState>>,
                Query(mut params):  Query<Q>) -> Result<Json<Vec<SearchResult>>, StatusCode> { ;
    tracing::info!("search params: {:?}", params);
    let res = SearchResult {
        result: format!("You searched for: {:?}", params.q),
    };
    //let res = arc_state.db_client.lock().await.search(params.q).await.map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(vec![res]))
}

#[derive(serde::Serialize)]
struct HealthStatus {
    status: String,
}

async fn health_check() -> Result<Json<HealthStatus>, StatusCode> {
    let data = HealthStatus {
        status: "OK".to_string(),
    };

    // Debes envolver el Json dentro de Ok()
    Ok(Json(data))
}


#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .route("/search", get(search))
        .route("/reports", post(post_reports))
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

