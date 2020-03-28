use rocket::request::{Outcome, FromRequest};
use rocket::Outcome::Success;
use rocket::Request;
use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::coll::options::{FindOneAndUpdateOptions};
use mongodb::db::ThreadedDatabase;

use crate::entities::{User, Channel};

#[derive(Clone)]
pub struct UsersRepository {
    host: String,
    port: u16,
}

impl UsersRepository {
    pub fn new(host: String, port: u16) -> UsersRepository {
        UsersRepository{host:host, port: port}

    }
    pub fn create (self, user: User) {
        let client = Client::connect(self.host.as_str(), self.port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("users");
        let chan = doc!{
            "name": user.idname,
            "address": user.idaddress,
        };
        coll.insert_one(chan.clone(), None).ok().expect("Failed to insert document");
    }

    pub fn put (self, user: User) {
        let client = Client::connect(self.host.as_str(), self.port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("users");
        let _update = doc!{
            "name": user.idname.clone(),
        };
        let _filter = doc!{
            "name": user.idname.clone(),
        };

        let options = FindOneAndUpdateOptions::new();
        let _ = coll.find_one_and_update(_filter, _update, Some(options));
    }



    pub fn get (self, id: String) -> Option<User>{
        let client = Client::connect(self.host.as_str(), self.port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("users");
        let user = doc!{
            "user": id,
        };
        let mut cursor = coll.find(Some(user.clone()), None)
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
        Success(UsersRepository{host: "".to_string(),port: 0})
    }
}

pub struct ChannelsRepository {
    host: String,
    port: u16,
}

impl ChannelsRepository {
    pub fn new(host: String, port: u16) -> ChannelsRepository {
        ChannelsRepository{host: host, port: port}
    }

    pub fn create (self, channel: Channel) {
        let client = Client::connect(self.host.as_str(), self.port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("channels");
        let bson_users: Vec<Bson> = channel.users.into_iter().map(|x| Bson::String(x)).collect(); 
        let chan = doc!{
            "name": channel.name,
            "users": bson_users,
        };
        coll.insert_one(chan.clone(), None).ok().expect("Failed to insert document");
    }

    pub fn put (self, channel: Channel) {
        let client = Client::connect(self.host.as_str(), self.port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("channels");
        let bson_users: Vec<Bson> = channel.users.into_iter().map(|x| Bson::String(x)).collect(); 
        let _update = doc!{
            "name": channel.name.clone(),
            "users": bson_users,
        };
        let _filter = doc!{
            "name": channel.name.clone(),
        };

        let options = FindOneAndUpdateOptions::new();
        let _ = coll.find_one_and_update(_filter, _update, Some(options));
    }

    pub fn get (self, id: String) -> Option<Channel> {
        let client = Client::connect(self.host.as_str(), self.port).expect("Failed to initialize standalone client.");
        let coll = client.db("test").collection("channels");
        let chan = doc!{
            "name": id,
        };
        let mut cursor = coll.find(Some(chan.clone()), None)
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
       Success(ChannelsRepository{host: "".to_string(), port: 0}) 
    }
}

