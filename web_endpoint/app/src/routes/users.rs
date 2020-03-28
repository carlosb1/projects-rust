use rocket_contrib::json::{Json};
use crate::db::UsersRepository;
use crate::usecases::{GetUser, NewUser, UpdateUser};
use crate::entities::User;



#[get("/users/<tag>")]
pub fn get_user_info(db: UsersRepository, tag: String) -> Json<User> {
    let _user = GetUser::new(Box::new(db)).run(tag); 
    Json(_user.unwrap())
}

#[post("/users", format="application/json", data="<user>")]
pub fn post_new_user(db: UsersRepository, user: User) -> Json<&'static str> {
    NewUser::new(Box::new(db)).run(user);
    Json("{'result': 'ok'}")

}

#[put("/users", format="application/json", data="<user>")]
pub fn put_user(db: UsersRepository, user: User) -> Json<&'static str> {
    UpdateUser::new(Box::new(db)).run(user);
    Json("{'result': 'ok'}")
}
