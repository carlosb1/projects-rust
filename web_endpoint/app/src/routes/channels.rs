use rocket_contrib::json::{Json};

use crate::db::ChannelsRepository;
use crate::usecases::GetChannel;

#[derive(Serialize)]
pub struct IdUser {
    pub identifier: String,
}


#[get("/channels/<tag>")]
pub fn get_user(db_channels: ChannelsRepository, tag: String) -> Json<Vec<IdUser>> {
    GetChannel::new(db_channels).run(tag); 
    let mut users: Vec<IdUser> = Vec::new();
    Json(users)
}
