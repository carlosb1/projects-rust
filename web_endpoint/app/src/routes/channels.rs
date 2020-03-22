use rocket_contrib::json::{Json};

use crate::db::ChannelsRepository;

#[derive(Serialize)]
pub struct IdUser {
    pub identifier: String,
}


#[get("/channels/<tag>")]
pub fn get_user(db_channels: ChannelsRepository, tag: String) -> Json<Vec<IdUser>> {
    let mut users: Vec<IdUser> = Vec::new();
    Json(users)
}
