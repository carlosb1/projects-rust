use rocket_contrib::json::{Json};

use crate::entities::Channel;
use crate::db::ChannelsRepository;
use crate::usecases::{GetChannel, NewChannel};

#[get("/channels/<tag>")]
pub fn get_user(db_channels: ChannelsRepository, tag: String) -> Json<Channel> {
    let _channel = GetChannel::new(db_channels).run(tag); 
    Json(_channel.unwrap())
}

#[post("/channels", format="application/json", data="<channel>")]
pub fn post_channel(db_channels: ChannelsRepository, channel: Channel) -> Json<&'static str> {
    NewChannel::new(Box::new(db_channels)).run(channel);
    Json("{'result': 'ok'}")
}
