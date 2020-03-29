#![feature(rustc_private)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate serde_derive;
extern crate mongodb;
extern crate rocket_contrib;
extern crate serde_json;

mod db;
mod entities;

use crate::db::{UsersRepository};
use crate::entities::User;

fn main() {
    let _host = "0.0.0.0".to_string();
    let _port = 27017;
    let user_repository = Box::new(UsersRepository::new(_host, _port));
    user_repository.clone().create(User{idname: "idname".to_string(), idaddress:"idaddress".to_string()});
    let restored_user = user_repository.clone().get("idname".to_string());
    println!("{restored_user}", restored_user=restored_user.unwrap());
    user_repository.clone().put(User{idname: "idname".to_string(), idaddress:"idaddress22".to_string()});

    
}
