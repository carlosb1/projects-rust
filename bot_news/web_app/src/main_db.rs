use futures::stream::StreamExt;
use mongodb::bson::Array;
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use std::error::Error;

//#[derive(Serialize, Deserialize, Clone)]
#[derive(Debug, Clone)]
pub struct News {
    id: String,
    link: String,
    title: String,
    descrip: String,
    data_ml: Array,
}
impl News {
    fn new(id: &str, link: &str, title: &str, descrip: &str, data_ml: Array) -> News {
        News {
            id: id.to_string(),
            link: link.to_string(),
            title: title.to_string(),
            descrip: descrip.to_string(),
            data_ml: data_ml,
        }
    }
}

async fn run_db() -> Result<Vec<News>, Box<dyn Error>> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options)?;
    let db = client.database("db_news");
    let collection = db.collection("news");
    let mut cursor = collection
        .find(None, FindOptions::builder().build())
        .await?;

    let mut values: Vec<News> = Vec::new();
    let empty_array = Array::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(doc) => {
                let id = doc.get_str("id").unwrap_or("");
                let link = doc.get_str("link").unwrap_or("");
                let title = doc.get_str("title").unwrap_or("");
                let descrip = doc.get_str("description").unwrap_or("");
                let data_ml = doc.get_array("data_ml").unwrap_or(&empty_array);
                let cloned_data = data_ml.to_owned();
                let news = News::new(id, link, title, descrip, cloned_data);
                values.push(news.clone());
            }
            Err(e) => println!("{}", e),
        }
    }
    Ok(values)
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let fut = run_db();
    let values = rt.block_on(fut);
    println!("{:?}", values);
    Ok(())
}
