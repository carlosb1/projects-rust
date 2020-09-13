use entities::User;
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
        let client_options = ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port))
            .await
            .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("users");
        let new_user = doc! {"id": user.id, "name": user.name, "password": user.password};
        let val = collection.insert_one(new_user, None).await?;
        match val {
            Some(_) => Some(()),
            None => None,
        }
    }

    pub async fn find_one(self, id: &str) -> Result<User, Box<dyn Error>> {
        let client_options = ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port))
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
                    let user = doc.get_str("link").unwrap_or("");
                    let password = doc.get_str("title").unwrap_or("");
                    let user = User::new(id, user, password);
                    return Ok(user);
                }
                Err(e) => println!("{}", e),
            }
        }
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
