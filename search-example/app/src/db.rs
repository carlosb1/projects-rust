use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::SearchPointsBuilder;
use crate::models::{EntryResult, SearchResult};

pub static COLLECTION: &str = "boe_disposiciones";
pub static LIMIT:  u64 = 20;



pub struct Client {
    client: Qdrant,
    pub model: TextEmbedding,
}


impl Client {
    pub fn new(qdrant_host: &str, qdrant_port: u16, api_key: Option<String>) -> anyhow::Result<Client> {
        tracing::debug!("Downloading model for embeddings");
        let mut model = TextEmbedding::try_new(
            InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
        )?;

        let mut client = Qdrant::from_url(format!("http://{qdrant_host}:{qdrant_port}").as_str());
        if api_key.is_some() {
            client = client.api_key(api_key.unwrap().as_str());
        }
        let client = client.build()?;
        Ok(Client{client, model})
    }


     pub async fn search(&mut self, query: Vec<String>) -> anyhow::Result<Vec<SearchResult>> {
        let mut search_results = Vec::new();
        let embed = self.model.embed(query, None)?;
        for x in embed {
            let embed = x.as_slice();
            let search_request = SearchPointsBuilder::new(
                COLLECTION,    // Collection name
                embed, // Search vector
                LIMIT,                  // Search limit, number of results to return
            ).with_payload(true);
            let results = self.client.search_points(search_request).await?;
            let time: f64 = results.time;
            let mut entries_to_return = Vec::<EntryResult>::new();
            for x in results.result {
                let score: f32 = x.score;

                let mut payloads_to_return = Vec::new();
                for (key,val) in x.payload.iter() {
                    let k = key.to_string();
                    let v = val.to_string();
                    payloads_to_return.push((k,v));
                }
                entries_to_return.push(EntryResult{score,payload: payloads_to_return});
            }
            let s_result = SearchResult{time, results: entries_to_return };
            search_results.push(s_result);
        }
        Ok(search_results)

    }
}

