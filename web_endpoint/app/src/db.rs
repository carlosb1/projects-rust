use rocket::request::{Outcome, FromRequest};
use rocket::Outcome::Success;
use rocket::Request;
use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::coll::Collection;
use mongodb::db::ThreadedDatabase;

use crate::entities::{User, Channel};

pub struct UsersRepository {
    coll: Option<Collection>,
}

impl UsersRepository {
    pub fn new(host: String, port: u32) -> UsersRepository {
        let client = Client::connect(host, port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("users");
        UsersRepository{coll:Some(coll)}

    }
    pub fn create (self, user: User) {
    }

    pub fn get (self, id: String) -> User  {
        User {}
    }    
}
impl<'a, 'r> FromRequest<'a, 'r> for UsersRepository  {
    type Error = ();
    fn from_request (_request: &'a Request<'r>) -> Outcome<Self, Self::Error>  {
        let _host = "0.0.0.0".to_string();
        Success(UsersRepository{host: _host, port:27017, coll: None}) 
    }
}

pub struct ChannelsRepository {
    host: String,
    port: u32,
}

impl ChannelsRepository {
    pub fn new(host: String, port: u32) -> ChannelsRepository {
        ChannelsRepository{host: host, port: port} 
    }

    pub fn create (self, channel: Channel) { 
    }

    pub fn get (self, id: String) -> Channel  {
        Channel {}
    }   
}
impl<'a, 'r> FromRequest<'a, 'r> for ChannelsRepository  {
    type Error = ();
    fn from_request (_request: &'a Request<'r>) -> Outcome<Self, Self::Error>  {
       let _host = "0.0.0.0".to_string();
       Success(ChannelsRepository{host: _host, port: 27017}) 
    }
}

