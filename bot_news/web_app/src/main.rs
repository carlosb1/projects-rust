#![feature(rustc_private)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate rocket_contrib;
extern crate serde_derive;
extern crate serde_json;
extern crate tera;

mod db;
mod entities;
mod routes;
mod utils;
use crate::routes::{errors, new, static_files};

// tera
use rocket_contrib::templates::Template;

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(Template::fairing())
        .mount(
            "/",
            routes![
                static_files::file,
                new::index,
                new::single_page_app,
                new::main,
            ],
        )
        .register(catchers![errors::not_found])
}

fn main() {
    rocket().launch();
}
