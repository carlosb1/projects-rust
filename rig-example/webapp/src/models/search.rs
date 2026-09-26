use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct SearchResult {
    pub title: String,
    pub excerpt: String,
    pub source: String,
    pub score: f32,
}


#[derive(Clone, Debug, Deserialize)]
pub struct SearchResponse {
    pub items: Vec<SearchResult>,
    pub page: usize,
    pub total_pages: usize,
    pub total: usize,
}