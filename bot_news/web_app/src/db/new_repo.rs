use crate::entities::News;
use futures::stream::StreamExt;
use mongodb::bson::{doc, Array, Bson};
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use rocket::request::{FromRequest, Outcome};
use rocket::Outcome::Success;
use rocket::Request;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Clone)]
pub struct NewsRepository {
    host: String,
    port: u16,
}

impl NewsRepository {
    pub fn new(host: String, port: u16) -> NewsRepository {
        NewsRepository { host, port }
    }
    pub async fn update(self, news: News) -> Option<()> {
        let client_options =
            ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port).as_str())
                .await
                .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("news");
        let _update = doc! {"$set" : { "id": news.id.clone(), "link": news.link, "title": news.title, "descrip": news.descrip, "approved": news.approved, "liked": news.liked, "fake": news.fake, "tags": news.tags}};
        let _filter = doc! {
            "id": news.id.clone()
        };
        let val = collection
            .find_one_and_update(_filter, _update, None)
            .await
            .expect("It was a problem to get result find and update");
        match val {
            Some(_) => Some(()),
            None => None,
        }
    }

    pub async fn find_one(self, id: &str) -> Option<News> {
        let client_options =
            ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port).as_str())
                .await
                .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("news");
        let mut cursor = collection
            .find(None, FindOptions::builder().build())
            .await
            .expect("It was not possible to get the cursor");

        let empty_array = Array::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    let id = doc.get_str("id").unwrap_or("");
                    let link = doc.get_str("link").unwrap_or("");
                    let title = doc.get_str("title").unwrap_or("");
                    let descrip = doc.get_str("description").unwrap_or("");
                    let approved = doc.get_i64("approved").unwrap_or(0);
                    let liked = doc.get_i64("liked").unwrap_or(0);
                    let fake = doc.get_i64("fake").unwrap_or(0);
                    let data_ml = doc.get_array("data_ml").unwrap_or(&empty_array);
                    let tags = doc
                        .get_array("tags")
                        .unwrap_or(&Vec::new())
                        .into_iter()
                        .map(Bson::from)
                        .map(|x| x.as_str().unwrap().to_string())
                        .collect::<Vec<String>>();
                    let cloned_data = data_ml.to_owned();
                    let new = News::new(
                        id,
                        link,
                        title,
                        descrip,
                        cloned_data,
                        approved,
                        liked,
                        fake,
                        tags,
                    );
                    return Some(new);
                }
                Err(e) => println!("{}", e),
            }
        }
        None
    }

    pub async fn all(self) -> Result<Vec<News>, Box<dyn Error>> {
        let client_options =
            ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port).as_str())
                .await
                .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("news");
        let mut cursor = collection
            .find(None, FindOptions::builder().build())
            .await
            .expect("It was not possible to get the cursor");

        let mut values: Vec<News> = Vec::new();
        let empty_array = Array::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    let id = doc.get_str("id").unwrap_or("");
                    let link = doc.get_str("link").unwrap_or("");
                    let title = doc.get_str("title").unwrap_or("");
                    let descrip = doc.get_str("description").unwrap_or("");
                    let approved = doc.get_i64("approved").unwrap_or(0);
                    let liked = doc.get_i64("liked").unwrap_or(0);
                    let fake = doc.get_i64("fake").unwrap_or(0);
                    let data_ml = doc.get_array("data_ml").unwrap_or(&empty_array);
                    let tags = doc
                        .get_array("tags")
                        .unwrap_or(&Vec::new())
                        .into_iter()
                        .map(Bson::from)
                        .map(|x| x.as_str().unwrap().to_string())
                        .collect::<Vec<String>>();
                    let cloned_data = data_ml.to_owned();
                    let news = News::new(
                        id,
                        link,
                        title,
                        descrip,
                        cloned_data,
                        approved,
                        liked,
                        fake,
                        tags,
                    );
                    values.push(news.clone());
                }
                Err(e) => println!("{}", e),
            }
        }
        Ok(values)
    }
    pub async fn find_by_ids(self, ids: Vec<String>) -> Result<Vec<News>, Box<dyn Error>> {
        let client_options =
            ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port).as_str())
                .await
                .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("news");
        let _filter = doc! {
            "id": {"$in": ids.clone() }
        };

        let mut cursor = collection
            .find(_filter, FindOptions::builder().build())
            .await
            .expect("It was not possible to get the cursor");

        let mut values: Vec<News> = Vec::new();
        let empty_array = Array::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    let id = doc.get_str("id").unwrap_or("");
                    let link = doc.get_str("link").unwrap_or("");
                    let title = doc.get_str("title").unwrap_or("");
                    let descrip = doc.get_str("description").unwrap_or("");
                    let approved = doc.get_i64("approved").unwrap_or(0);
                    let liked = doc.get_i64("liked").unwrap_or(0);
                    let fake = doc.get_i64("fake").unwrap_or(0);
                    let data_ml = doc.get_array("data_ml").unwrap_or(&empty_array);
                    let tags = doc
                        .get_array("tags")
                        .unwrap_or(&Vec::new())
                        .into_iter()
                        .map(Bson::from)
                        .map(|x| x.as_str().unwrap().to_string())
                        .collect::<Vec<String>>();
                    let cloned_data = data_ml.to_owned();
                    let news = News::new(
                        id,
                        link,
                        title,
                        descrip,
                        cloned_data,
                        approved,
                        liked,
                        fake,
                        tags,
                    );
                    values.push(news.clone());
                }
                Err(e) => println!("{}", e),
            }
        }
        Ok(values)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for NewsRepository {
    type Error = ();
    fn from_request(_request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        Success(NewsRepository {
            host: "".to_string(),
            port: 0,
        })
    }
}
