#![feature(proc_macro_hygiene)]

use rocket::response::NamedFile;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::io;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InternalServerError,
}

#[get("/index")]
pub fn index() -> Template {
    let context: HashMap<&str, &str> = [("name", "Carlos")].iter().cloned().collect();
    Template::render("main", &context)
}

#[get("/share")]
pub fn single_page_app() -> io::Result<NamedFile> {
    NamedFile::open("static/build/index.html")
}

#[get("/main")]
pub fn main() -> Template {
    let context: HashMap<&str, &str> = [("name", "Carlos")].iter().cloned().collect();
    Template::render("main", &context)
}
