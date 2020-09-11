#![feature(proc_macro_hygiene)]

use rocket::response::status;
use rocket::response::NamedFile;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::io;

//TODO change name endpoint
//TODO add logging

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

#[post("/<userid>/fake/<articleid>", format = "application/json")]
pub fn point_fake(userid: String, articleid: String) -> status::Accepted<String> {
    status::Accepted(Some("{'result':'ok'}".to_string()))
}

#[post("/<userid>/new_comment/<articleid>", format = "application/json")]
pub fn new_comment(userid: String, articleid: String) -> status::Accepted<String> {
    status::Accepted(Some("{'result':'ok'}".to_string()))
}

#[post("/<userid>/approve/<articleid>", format = "application/json")]
pub fn point_approve(userid: String, articleid: String) -> status::Accepted<String> {
    status::Accepted(Some("{'result':'ok'}".to_string()))
}

#[post("/<userid>/like/<articleid>", format = "application/json")]
pub fn point_like(userid: String, articleid: String) -> status::Accepted<String> {
    status::Accepted(Some("{'result':'ok'}".to_string()))
}
