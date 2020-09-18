use mongodb::bson::Array;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct News {
    pub id: String,
    pub link: String,
    pub title: String,
    pub descrip: String,
    pub data_ml: Array,
}
impl News {
    pub fn new(id: &str, link: &str, title: &str, descrip: &str, data_ml: Array) -> News {
        News {
            id: id.to_string(),
            link: link.to_string(),
            title: title.to_string(),
            descrip: descrip.to_string(),
            data_ml: data_ml,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub password: String,
}

impl User {
    pub fn new(id: &str, name: &str, password: &str) -> User {
        User {
            id: id.to_string(),
            name: name.to_string(),
            password: password.to_string(),
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
