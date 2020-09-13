#[derive(Serialize, Deserialize, Debug, Clone)]
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

pub struct User {
    id: String,
    name: String,
    password: String,
}

impl User {
    fn new(id: &str, name: &str, password: &str) -> User {
        User {
            id: id.to_string(),
            name: name.to_string(),
            password: password.to_string(),
        }
    }
}

pub struct Comment {
    iduser: String,
    idnew: String,
    comment: String,
}

impl Comment {
    fn new(iduser: &str, idnew: &str, comment: &str) -> Comment {
        User {
            iduser: iduser.to_string(),
            idnew: idnew.to_string(),
            comment: comment.to_string(),
        }
    }
}
