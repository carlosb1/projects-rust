use rocket_contrib::json::{Json};

use crate::entities::Channel;
use crate::db::ChannelsRepository;
use crate::usecases::{GetChannel, NewChannel, UpdateChannel};
use std::env;

#[get("/channels/<tag>")]
pub fn get_user(tag: String) -> Json<Channel> {
    let db_host  = env::var("MONGODB_HOST").unwrap_or("0.0.0.0".to_string());
    let db_channels = ChannelsRepository::new(db_host, 27017);    
    let _channel = GetChannel::new(db_channels).run(tag); 
    Json(_channel.unwrap())
}

#[post("/channels", format="application/json", data="<channel>")]
pub fn post_channel(channel: Channel) -> Json<&'static str> {
    let db_host  = env::var("MONGODB_HOST").unwrap_or("0.0.0.0".to_string());
    let db_channels = ChannelsRepository::new(db_host, 27017);    
    NewChannel::new(db_channels).run(channel);
    Json("{'result': 'ok'}")
}

#[put("/channels", format="application/json", data="<channel>")]
pub fn put_channel(channel: Channel) -> Json<&'static str> {
    let db_host  = env::var("MONGODB_HOST").unwrap_or("0.0.0.0".to_string());
    let db_channels = ChannelsRepository::new(db_host, 27017);    
    UpdateChannel::new(db_channels).run(channel);
    Json("{'result': 'ok'}")
}
