use entities::Comment;
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
pub struct CommentRepository {
    host: String,
    port: u16,
}

impl CommentRepository {
    pub fn new(host: String, port: u16) -> CommentRepository {
        CommentRepository { host, port }
    }
    pub async fn insert_one(self, comment: Comment) -> Option<()> {
        let client_options = ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port))
            .await
            .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("comments");
        let new_comment =
            doc! {"iduser": comment.iduser, "idnew": comment.idnew, "comment": comment.comment};
        let val = collection.insert_one(new_user, None).await?;
        match val {
            Some(_) => Some(()),
            None => None,
        }
    }

    pub async fn find_by_new_id(self, idnew: &str) -> Result<Vec<Comment>, Box<dyn Error>> {
        let client_options = ClientOptions::parse(format!("mongodb://{}:{}", self.host, self.port))
            .await
            .expect("It was not possible to set up the client");
        let client =
            Client::with_options(client_options).expect("It was not possible to set up options");
        let collection = client.database("db_news").collection("comments");
        let mut cursor = collection
            .find(doc! {"idnew", idnew}, FindOptions::builder().build())
            .await
            .expect("It was not possible to get the cursor");

        let mut values: Vec<Comment> = Vec::new();
        let empty_array = Array::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => {
                    let iduser = doc.get_str("iduser").unwrap_or("");
                    let idnew = doc.get_str("idnew").unwrap_or("");
                    let comment = doc.get_str("comment").unwrap_or("");
                    let new_comment = Comment::new(iduser, idnew, comment);
                    values.push();
                }
                Err(e) => println!("{}", e),
            }
        }
        Ok(values)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for CommentRepository {
    type Error = ();
    fn from_request(_request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        Success(CommentRepository {
            host: "".to_string(),
            port: 0,
        })
    }
}
