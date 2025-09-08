use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResult {
    pub time: f64,
    pub results: Vec<EntryResult>
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EntryResult {
    pub score: f32,
    pub payload: Vec<(String, String)>
}