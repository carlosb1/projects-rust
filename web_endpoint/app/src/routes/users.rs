use rocket_contrib::json::{Json};
use crate::db::UsersRepository;
use crate::usecases::GetUser;

#[derive(Serialize)]
pub struct User {
    pub address: String,
}


#[get("/users/<tag>")]
pub fn get_user_info(db: UsersRepository, tag: String) -> Json<User> {
    let _user = GetUser::new(Box::new(db)).run(tag);
    Json(User{address: "testuser".to_string()})
}
