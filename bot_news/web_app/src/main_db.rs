use futures::stream::StreamExt;
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use std::error::Error;

async fn run_db() -> Result<(), Box<dyn Error>> {
    println!("Clieent db");
    let client_options = ClientOptions::parse("mongodb://localhost:27017")
        .await
        .unwrap();
    let client = Client::with_options(client_options)?;
    let db = client.database("db_news");
    let collection = db.collection("news");
    let mut cursor = collection
        .find(None, FindOptions::builder().build())
        .await?;

    while let Some(doc) = cursor.next().await {
        println!("allo presidente");
        println!("{}", doc.unwrap());
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let fut = run_db();
    rt.block_on(fut);
    Ok(())
}
