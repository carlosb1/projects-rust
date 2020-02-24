#![feature(rustc_private)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate tera;

mod routes;
use crate::routes::{ static_files, get, errors };

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
                get::get_search,
            ],
        )
        .register(catchers![errors::not_found])
}

fn main() {
    rocket().launch();
}


