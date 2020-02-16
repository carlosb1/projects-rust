use std::io;
use rocket::response::{NamedFile};
use rocket_contrib::json::{Json, JsonValue};

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InternalServerError,
}

#[derive(Serialize, Deserialize)]
pub struct Search {
}

#[get("/")]
pub fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

#[get("/share")]
pub fn single_page_app() -> io::Result<NamedFile> {
    NamedFile::open("static/build/index.html")
}

#[get("/searches/<tag>", format="application/json")]
pub fn get_search(tag: String) -> Result<Json<Vec<Search>>, ApiError> {
}

