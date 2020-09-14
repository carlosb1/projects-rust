#![feature(proc_macro_hygiene)]

use rocket::response::status;
use rocket::response::NamedFile;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::io;

use crate::db::comment_repo::CommentRepository;
use crate::db::new_repo::NewsRepository;
use crate::db::user_repo::UserRepository;
use crate::entities::Comment;
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
    let news_repo = NewsRepository::new("localhost".to_string(), 27017);

    //TODO how to manage this error
    // find by user?
    // let news = news_repo.all().await.unwrap();

    let context: HashMap<&str, &str> = [("name", "Carlos")].iter().cloned().collect();
    Template::render("main", &context)
}

#[post("/<userid>/new_comment/<articleid>", format = "application/json")]
pub fn new_comment(userid: String, articleid: String) -> status::Accepted<String> {
    // TODO add environment variable.
    let comment_repo = CommentRepository::new("localhost".to_string(), 27017);

    // TODO get json data information.
    let comment_info = "";
    comment_repo.insert_one(Comment::new(
        userid.as_str(),
        articleid.as_str(),
        comment_info,
    ));
    status::Accepted(Some("{'result':'ok'}".to_string()))
}
