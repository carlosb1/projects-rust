use std::fmt;

use rocket::Outcome::Success;
use rocket::Request;

use rocket::Data;
use rocket::data::{self, FromDataSimple};
use rocket::http::Status;
use rocket::Outcome::Failure;


#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(Clone)]    
pub struct User  {
    pub idname: String,
    pub idaddress: String,
}

impl FromDataSimple for User {
    type Error = String;
    
    #[allow(unused_variables)]
    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String> {
        let reader = data.open();
        match serde_json::from_reader(reader).map(|val| val) {
            Ok(value) => Success(value),
            Err(e) => Failure((Status::BadRequest, e.to_string())),
        }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.idname, self.idaddress)
    }
}


#[derive(Serialize)]
#[derive(Deserialize)]
#[derive(Clone)]    
pub struct Channel {
    pub name: String,
    pub users: Vec<String>,
}

impl FromDataSimple for Channel {
    type Error = String;
    
    #[allow(unused_variables)]
    fn from_data(req: &Request, data: Data) -> data::Outcome<Self, String> {
        let reader = data.open();
        match serde_json::from_reader(reader).map(|val| val) {
            Ok(value) => Success(value),
            Err(e) => Failure((Status::BadRequest, e.to_string())),
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {:?})", self.name, self.users)
    }
}
