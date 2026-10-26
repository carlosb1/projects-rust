use qdrant_client::Qdrant;
use qdrant_client::qdrant::SearchPointsBuilder;
use serde::{Deserialize, Serialize};
use rig_core::embeddings::EmbeddingModel as _;
use rig_fastembed::{
    Client as FastembedClient,
    EmbeddingModel,
    FastembedModel,
};


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
    pub model: EmbeddingModel,
    bucket_collection: String,
}

impl Client {
    pub fn new(qdrant_url: String, api_key: Option<String>, bucket_collection: String) -> anyhow::Result<Client> {
        tracing::debug!("Downloading model for embeddings");
        let model = FastembedClient::new()
            .embedding_model(&FastembedModel::AllMiniLML6V2)?;

        let mut client = Qdrant::from_url(qdrant_url.as_str());
        if api_key.is_some() {
            client = client.api_key(api_key.unwrap().as_str());
        }
        let client = client.build()?;
        Ok(Client{client, model, bucket_collection})
    }

    //TODO add pagination
    pub async fn search(&mut self, queries: Vec<String>) -> anyhow::Result<Vec<SearchResult>> {
        let mut search_results = Vec::new();
        let embeddings = self.model.embed_texts(queries).await?;
        //ADD option for ands and paging, intersection of values.
        for embedding in embeddings {
            let vector: Vec<f32> = embedding.vec
                .into_iter()
                .map(|value| value as f32)
                .collect();

            let search_request = SearchPointsBuilder::new(
                self.bucket_collection.clone(),
                vector,
                LIMIT,
            ).with_payload(true);
            let results = self.client.search_points(search_request).await?;
            let time: f64 = results.time;
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

