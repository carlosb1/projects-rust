#![feature(rustc_private)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
#[macro_use(bson, doc)] extern crate mongodb;
extern crate rocket_contrib;
extern crate serde_json;

mod entities;
mod routes;
mod db;
use crate::routes::{ static_files, get, users, channels};
// tera
use rocket_contrib::templates::Template;
use db::{UsersRepository, ChannelsRepository};


fn rocket() -> rocket::Rocket {
    let db_users = UsersRepository::new("0.0.0.0".to_string(), 27017);    
    let db_channels = ChannelsRepository{};    

    rocket::ignite()
        .manage(db_users).manage(db_channels)
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                static_files::file,
                get::index,
                get::single_page_app,
                users::get_user_info,
                channels::get_user,
            ],
        )
}

fn main() {
    rocket().launch();
}


