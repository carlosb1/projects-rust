use rocket_contrib::json::{Json};
use crate::db::UsersRepository;

#[derive(Serialize)]
pub struct User {
    pub address: String,
}


#[get("/users/<tag>")]
pub fn get_user_info(db: UsersRepository, tag: String) -> Json<User> {
    Json(User{address: "testuser".to_string()})
}
