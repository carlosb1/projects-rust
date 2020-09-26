#![feature(rustc_private)]
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate crypto;
extern crate rocket_contrib;
extern crate serde_derive;
extern crate serde_json;
extern crate tera;

mod db;
mod entities;
mod routes;
use crate::routes::{errors, new, static_files};

// tera
use db::user_repo::UserRepository;
use entities::User;
use new::{hash_password, load_mongo_credentials};
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
                new::new_comment,
                new::save_tags,
                new::fake,
                new::like,
                new::approve,
                new::login,
                new::search,
            ],
        )
        .register(catchers![errors::not_found])
}

fn main() {
    let (mongo_host, mongo_port) = load_mongo_credentials();
    let user_repo = UserRepository::new(mongo_host.clone(), mongo_port.clone());
    let new_user = User::new("0", "anonymous", hash_password(&"".to_string()).as_str());

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(user_repo.update(new_user));
    rocket().launch();
}
