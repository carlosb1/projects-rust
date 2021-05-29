use mongodb::bson::Array;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct News {
    pub id: String,
    pub link: String,
    pub title: String,
    pub descrip: String,
    pub data_ml: Array,
    pub approved: i64,
    pub liked: i64,
    pub fake: i64,
    pub tags: Vec<String>,
}
impl News {
    pub fn new(
        id: &str,
        link: &str,
        title: &str,
        descrip: &str,
        data_ml: Array,
        approved: i64,
        liked: i64,
        fake: i64,
        tags: Vec<String>,
    ) -> News {
        News {
            id: id.to_string(),
            link: link.to_string(),
            title: title.to_string(),
            descrip: descrip.to_string(),
            data_ml: data_ml,
            approved: approved,
            liked: liked,
            fake: fake,
            tags: tags,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub password: String,
    pub like_articles: Vec<String>,
    pub approved_articles: Vec<String>,
    pub fake_articles: Vec<String>,
}

impl User {
    pub fn new(id: &str, name: &str, password: &str) -> User {
        User {
            id: id.to_string(),
            name: name.to_string(),
            password: password.to_string(),
            like_articles: Vec::new(),
            approved_articles: Vec::new(),
            fake_articles: Vec::new(),
        }
    }
    pub fn new_with_articles(
        id: &str,
        name: &str,
        password: &str,
        like_articles: Vec<String>,
        approved_articles: Vec<String>,
        fake_articles: Vec<String>,
    ) -> User {
        User {
            id: id.to_string(),
            name: name.to_string(),
            password: password.to_string(),
            like_articles: like_articles,
            approved_articles: approved_articles,
            fake_articles: fake_articles,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Comment {
    pub iduser: String,
    pub idnew: String,
    pub comment: String,
}

impl Comment {
    pub fn new(iduser: &str, idnew: &str, comment: &str) -> Comment {
        Comment {
            iduser: iduser.to_string(),
            idnew: idnew.to_string(),
            comment: comment.to_string(),
        }
    }
}
