#![feature(rustc_private)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate serde_derive;
extern crate mongodb;
extern crate rocket_contrib;
extern crate serde_json;

mod entities;
mod routes;
mod db;
mod usecases;

use std::env;
use crate::routes::{ static_files, get, users, channels};
// tera
use rocket_contrib::templates::Template;
use db::{UsersRepository, ChannelsRepository};


fn rocket() -> rocket::Rocket {

    let db_host  = env::var("MONGODB_HOST").unwrap_or("0.0.0.0".to_string());

    println!("-> DB host: {}", db_host);

    let db_users = UsersRepository::new(db_host.clone(), 27017);    
    let db_channels = ChannelsRepository::new(db_host.clone(), 27017);    

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
                users::post_new_user,
                users::put_user,
                channels::get_user,
                channels::post_channel,
                channels::put_channel,
            ],
        )
}

fn main() {
    rocket().launch();
}


