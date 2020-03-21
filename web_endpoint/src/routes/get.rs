#![feature(proc_macro_hygiene)]

use std::io;
use rocket::response::{NamedFile};
use rocket_contrib::json::{Json, JsonValue};


#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InternalServerError,
}

#[derive(Serialize)]
pub struct Search {
    pub result: String,
}

#[get("/index")]
pub fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/share")]
pub fn single_page_app() -> io::Result<NamedFile> {
    NamedFile::open("static/build/index.html")
}
