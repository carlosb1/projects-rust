use rocket::request::{Outcome, FromRequest};
use rocket::Outcome::Success;
use rocket::Request;

use crate::entities::{User, Channel};

pub struct UsersRepository {
    host: String, 
    port: u32,
}

impl UsersRepository {
    pub fn new(host: String, port: u32) -> UsersRepository {
        UsersRepository{host: host, port:port}
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
        //let _host = env::var("MONGODB_ADDRESS").unwrap_or_default("0.0.0.0".to_string())
        let _host = "0.0.0.0".to_string();
        Success(UsersRepository{host: _host, port:27017}) 
    }
}

pub struct ChannelsRepository;

impl ChannelsRepository {
    pub fn create (self, channel: Channel) {
    
    }

    pub fn get (self, id: String) -> Channel  {
        Channel {}
    }   
}
impl<'a, 'r> FromRequest<'a, 'r> for ChannelsRepository  {
    type Error = ();
    fn from_request (_request: &'a Request<'r>) -> Outcome<Self, Self::Error>  {
       Success(ChannelsRepository{}) 
    }
}

