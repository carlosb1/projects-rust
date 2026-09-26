use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SearchRequest {
    pub query: String,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Deserialize)]
pub struct SearchResponse {
    pub items: Vec<SearchResult>,
    pub page: usize,
    pub total_pages: usize,
    pub total: usize,
}

use gloo_net::http::Request;
use crate::models::search::SearchResult;

pub async fn search(
    query: &str,
    page: usize,
) -> Result<SearchResponse, String> {
    let query = urlencoding::encode(query);

    let url = format!(
        "/api/search?q={query}&page={page}&page_size=10"
    );

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.ok() {
        return Err(format!(
            "Search failed: {}",
            response.status()
        ));
    }

    response
        .json()
        .await
        .map_err(|e| e.to_string())
}