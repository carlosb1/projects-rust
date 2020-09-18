use crate::entities::User;
use futures::stream::StreamExt;
use mongodb::bson::doc;
use mongodb::bson::Array;
use mongodb::{options::ClientOptions, options::FindOptions, Client};
use rocket::request::{FromRequest, Outcome};
use rocket::Outcome::Success;
use rocket::Request;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Clone)]
pub struct UserRepository {
    host: String,
    port: u16,
}

impl UserRepository {
    pub fn new(host: String, port: u16) -> UserRepository {
        UserRepository { host, port }
    }
    pub async fn insert_one(self, user: User) -> Option<()> {
        let client_options =
            ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port).as_str())
                .await
                .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("users");
        let _update = doc! {"$set" : {"id": user.id.clone(), "name": user.name, "password": user.password, "like_articles": user.like_articles, "approved_articles": user.approved_articles, "fake_articles": user.fake_articles}};
        let _filter = doc! {
            "id": user.id.clone()
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

    pub async fn find_one(self, id: &str) -> Option<User> {
        let client_options =
            ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port).as_str())
                .await
                .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("users");
        let mut cursor = collection
            .find(None, FindOptions::builder().build())
            .await
            .expect("It was not possible to get the cursor");

        let empty_array = Array::new();

        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    let id = doc.get_str("id").unwrap_or("");
                    let name = doc.get_str("name").unwrap_or("");
                    let password = doc.get_str("password").unwrap_or("");
                    let like_articless = doc
                        .get_array("like_articles")
                        .into_iter()
                        .map(|x| x.as_str().to_string())
                        .recv()
                        .collect()
                        .unwrap_or(Vec::new());
                    let approved_articles = doc
                        .get_array("approved_articles")
                        .into_iter()
                        .map(|x| x.as_str().to_string())
                        .recv()
                        .collect()
                        .unwrap_or(Vec::new());
                    let fake_articles = doc
                        .get_array("fake_articles")
                        .into_iter()
                        .map(|x| x.as_str().to_string())
                        .recv()
                        .collect()
                        .unwrap_or(Vec::new());
                    let user = User::new_with_articles(
                        id,
                        user,
                        password,
                        like_articles,
                        approved_articles,
                        fake_articles,
                    );
                    return Some(user);
                }
                Err(e) => println!("{}", e),
            }
        }
        None
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for UserRepository {
    type Error = ();
    fn from_request(_request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        Success(UserRepository {
            host: "".to_string(),
            port: 0,
        })
    }
}
