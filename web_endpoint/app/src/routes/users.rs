use rocket_contrib::json::{Json};
use crate::db::UsersRepository;
use crate::usecases::{GetUser, NewUser, UpdateUser};
use crate::entities::User;
use std::env;

#[get("/users/<tag>")]
pub fn get_user_info(tag: String) -> Json<User> {
    let db_host  = env::var("MONGODB_HOST").unwrap_or("0.0.0.0".to_string());
    let db_users = UsersRepository::new(db_host, 27017);    
    let _user = GetUser::new(db_users).run(tag); 
    Json(_user.unwrap())
}

#[post("/users", format="application/json", data="<user>")]
pub fn post_new_user(user: User) -> Json<&'static str> {
    let db_host  = env::var("MONGODB_HOST").unwrap_or("0.0.0.0".to_string());
    let db_users = UsersRepository::new(db_host, 27017);    
    println!(" Posting user {}", user);
    NewUser::new(db_users).run(user);
    Json("{'result': 'ok'}")

}

#[put("/users", format="application/json", data="<user>")]
pub fn put_user(user: User) -> Json<&'static str> {
    let db_host  = env::var("MONGODB_HOST").unwrap_or("0.0.0.0".to_string());
    let db_users = UsersRepository::new(db_host, 27017);    
    UpdateUser::new(db_users).run(user);
    Json("{'result': 'ok'}")
}
