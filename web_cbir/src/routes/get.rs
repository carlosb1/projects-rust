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

#[get("/searches/<tag>")]
pub fn get_search(tag: String) -> Json<Vec<Search>> {

    let mut searches: Vec<Search> = Vec::new();
    searches.push(Search{result: "hello".to_string()});
    searches.push(Search{result: "hello2".to_string()});
    searches.push(Search{result: "hello3".to_string()});
    searches.push(Search{result: "hello4".to_string()});
    Json(searches)
}

