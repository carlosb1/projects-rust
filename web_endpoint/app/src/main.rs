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

use crate::routes::{ static_files, get, users, channels};
// tera
use rocket_contrib::templates::Template;


fn rocket() -> rocket::Rocket {
    rocket::ignite()
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


