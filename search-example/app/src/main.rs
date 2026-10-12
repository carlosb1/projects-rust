mod db;
mod models;

use crate::models::SearchResult;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, get_service};
use axum::{Json, Router};
use axum_extra::extract::Query;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::{Mutex, broadcast};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, info};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

struct AppState {
    tx: Sender<String>,
    db_client: Arc<Mutex<db::Client>>,
}

#[derive(Debug, Deserialize)]
struct Q {
    q: Vec<String>,
}

//TODO add endpoints for agents
async fn search(
    State(arc_state): State<Arc<AppState>>,
    Query(mut params): Query<Q>,
) -> Result<Json<Vec<SearchResult>>, StatusCode> {
    tracing::info!("search params: {:?}", params);
    let res = arc_state
        .db_client
        .lock()
        .await
        .search(params.q)
        .await
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(res))
}

#[derive(Serialize)]
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //TODO open this port
    let qdrant_host = std::env::var("QDRANT_HOST").unwrap_or("0.0.0.0".to_string());
    let qdrant_port: u16 = std::env::var("QDRANT_PORT")
        .unwrap_or("6334".to_string())
        .parse()?;

    // tracing info
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set up application state for use with with_state().
    // let user_set = Mutex::new(HashSet::new());
    let (tx, _rx) = broadcast::channel(1024);
    let db_client = Arc::new(Mutex::new(db::Client::new(
        qdrant_host.as_str(),
        qdrant_port,
        None,
    )?));
    let app_state = Arc::new(AppState {
        tx: tx.clone(),
        db_client,
    });

    let static_dir = get_service(ServeDir::new("./dist")).handle_error(|err| async move {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("serve error: {err}"),
        )
    });

    //TODO Testing objective
    use tower_http::cors::CorsLayer;

    let app = Router::new()
        .route("/search", get(search))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .fallback_service(static_dir)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
