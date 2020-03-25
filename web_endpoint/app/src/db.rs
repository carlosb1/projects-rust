use rocket::request::{Outcome, FromRequest};
use rocket::Outcome::Success;
use rocket::Request;
use mongodb::{Bson, bson, doc};
use std::vec;
use mongodb::{Client, ThreadedClient};
use mongodb::coll::Collection;
use mongodb::db::ThreadedDatabase;

use crate::entities::{User, Channel};

pub struct UsersRepository {
    coll: Option<Collection>,
}

impl UsersRepository {
    pub fn new(host: String, port: u16) -> UsersRepository {
        let client = Client::connect(host.as_str(), port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("users");
        UsersRepository{coll:Some(coll)}

    }
    pub fn create (self, user: User) {
        let chan = doc!{
            "name": user.idname,
            "address": user.idaddress,
        };
        self.coll.unwrap().insert_one(chan.clone(), None).ok().expect("Failed to insert document");
    }

    pub fn get (self, id: String) -> Option<User>{
        let user = doc!{
            "user": id,
        };
        let mut cursor = self.coll.unwrap().find(Some(user.clone()), None)
        .ok().expect("Failed to execute find.");
        let result = match cursor.next() {
            Some(val) => {
                match val {
                    Ok(doc) => {
                        let _idname: String = doc.get_str("idname").unwrap_or("").to_string();
                        let _idaddress:  String = doc.get_str("idaddress").unwrap_or("").to_string();
                        Some(User{idname: _idname, idaddress: _idaddress})
                    },
                    Err(_) => None,
                }
            },
            None => None,
        };
        result
    }    
}
impl<'a, 'r> FromRequest<'a, 'r> for UsersRepository  {
    type Error = ();
    fn from_request (_request: &'a Request<'r>) -> Outcome<Self, Self::Error>  {
        let _host = "0.0.0.0".to_string();
        Success(UsersRepository{coll: None})
    }
}

pub struct ChannelsRepository {
    coll: Option<Collection>,
}

impl ChannelsRepository {
    pub fn new(host: String, port: u16) -> ChannelsRepository {
        let client = Client::connect(host.as_str(), port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("channels");
        ChannelsRepository{coll:Some(coll)}
    }

    pub fn create (self, channel: Channel) {
        let chan = doc!{
            "name": channel.name,
            "users": [],
        };
        self.coll.unwrap().insert_one(chan.clone(), None).ok().expect("Failed to insert document");
    }

    pub fn get (self, id: String) -> Option<Channel> {
        let chan = doc!{
            "name": "",
    
            "users": [],
        };
        let mut cursor = self.coll.unwrap().find(Some(chan.clone()), None)
        .ok().expect("Failed to execute find.");
        let result = match cursor.next() {
            Some(val) => {
                match val {
                    Ok(doc) => {
                        let _name: String = doc.get_str("name").unwrap_or("").to_string();
                        let _users: Vec<String> = doc.get_array("users").unwrap_or(&Vec::new()).into_iter().map(|x| x.to_string()).collect();
                        Some(Channel{name:_name, users: _users})
                    },
                    Err(_) => None,
                }
            },
            None => None,
        };
        result
    }   
}
impl<'a, 'r> FromRequest<'a, 'r> for ChannelsRepository  {
    type Error = ();
    fn from_request (_request: &'a Request<'r>) -> Outcome<Self, Self::Error>  {
       let _host = "0.0.0.0".to_string();
       Success(ChannelsRepository{coll:None}) 
    }
}

