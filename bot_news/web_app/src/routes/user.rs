#![feature(proc_macro_hygiene)]

use rocket::response::status;
use rocket::response::NamedFile;
use rocket_contrib::json::{Json, JsonValue};
use rocket_contrib::templates::Template;
use std::collections::HashMap;
use std::env;
use std::io;
use tera::Context;

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
    let MONGO_HOST: &str = env::var("MONGO_HOST")
        .unwrap_or("localhost".to_string())
        .as_str();
    let MONGO_PORT: u16 = env::var("MONGO_PORT")
        .unwrap_or("27017".to_string())
        .parse::<u16>()
        .expect("It is not a number.");
    let news_repo = NewsRepository::new("localhost".to_string(), 27017);

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    //TODO how to manage this error
    // find by user?
    let fut = news_repo.all();
    let news = rt.block_on(fut).unwrap_or(Vec::new());
    let mut context = Context::new();
    context.insert("news", &news);
    Template::render("main", &context)
}

#[post("/<userid>/new_comment/<articleid>", format = "application/json")]
pub fn new_comment(userid: String, articleid: String) -> status::Accepted<String> {
    let MONGO_HOST = env::var("MONGO_HOST")
        .unwrap_or("localhost".to_string())
        .as_str();
    let MONGO_PORT = env::var("MONGO_PORT")
        .unwrap_or("27017".to_string())
        .parse::<u16>()
        .expect("It is not a number.");
    // TODO add environment variable.
    let comment_repo = CommentRepository::new(MONGO_HOST.to_string().clone(), MONGO_PORT.clone());

    // TODO get json data information.
    let comment_info = "";
    comment_repo.insert_one(Comment::new(
        userid.as_str(),
        articleid.as_str(),
        comment_info,
    ));
    status::Accepted(Some("{'result':'ok'}".to_string()))
}
