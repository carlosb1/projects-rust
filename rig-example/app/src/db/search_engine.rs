use std::sync::Arc;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::SearchPointsBuilder;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

pub static LIMIT:  u64 = 20;


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

pub struct Client {
    client: Qdrant,
    pub model: Arc<Mutex<TextEmbedding>>,
    bucket_collection: String,
}

impl Client {
    pub fn new(qdrant_url: String, api_key: Option<String>, bucket_collection: String) -> anyhow::Result<Client> {
        tracing::info!("Downloading model for embeddings");
        let model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        )?;

        let mut client = Qdrant::from_url(qdrant_url.as_str());
        if api_key.is_some() {
            client = client.api_key(api_key.unwrap().as_str());
        }
        let client = client.build()?;
        Ok(Client{client, model: Arc::new(Mutex::new(model)), bucket_collection})
    }
    pub async fn count(&self) -> anyhow::Result<usize> {
        use qdrant_client::qdrant::CountPointsBuilder;

        let response = self.client
            .count(CountPointsBuilder::new(self.bucket_collection.as_str()).exact(true))
            .await?;

        let result = response.result
            .ok_or_else(|| anyhow::anyhow!("Qdrant doesn't return pags"))?;

        Ok(usize::try_from(result.count)?)
    }

    pub async fn search(&self, query: Vec<String>, page: usize, page_size: usize) -> anyhow::Result<Vec<SearchResult>> {
        anyhow::ensure!(page >= 1, "page debe ser mayor o igual a 1");
        anyhow::ensure!(page_size >= 1, "page_size debe ser mayor o igual a 1");

        let limit = u64::try_from(page_size)?;
        let offset = (page - 1)
            .checked_mul(page_size)
            .ok_or_else(|| anyhow::anyhow!("Too much pages and pagination"))?;
        let offset = u64::try_from(offset)?;

        let mut search_results = Vec::new();
        let embeddings = {
            let mut model = self.model.lock()
                .map_err(|_| anyhow::anyhow!("El mutex del modelo está envenenado"))?;
            model.embed(query, None)?
        };

        //ADD option for ands and paging, intersection of values.
        for embed in embeddings {
            let search_request = SearchPointsBuilder::new(
                self.bucket_collection.as_str(),    // Collection name
                embed, // Search vector
                limit,                  // Search limit, number of results to return
            ).with_payload(true)
            .offset(offset);
            let results = self.client.search_points(search_request).await?;
            let time: f64 = results.time;
            tracing::info!("Search results: {:?}", results.result.len());
            let mut entries_to_return = Vec::<EntryResult>::new();
            for x in results.result {
                let score: f32 = x.score;

                let mut payloads_to_return = Vec::new();
                for (key,val) in x.payload.iter() {
                    let k = key.to_string();
                    if let Some(v) = val.as_str()  {
                        payloads_to_return.push((k,v.clone()));
                    }
                }
                entries_to_return.push(EntryResult{score,payload: payloads_to_return});
            }
            let s_result = SearchResult{time, results: entries_to_return };
            search_results.push(s_result);
        }
        Ok(search_results)

    }
}

