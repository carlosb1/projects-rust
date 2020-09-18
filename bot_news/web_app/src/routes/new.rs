#![feature(proc_macro_hygiene)]

use log::info;
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
use crate::entities::{Comment, User};
use serde::Deserialize;

//TODO change name endpoint

#[derive(Deserialize, Clone)]
pub struct CommentDTO {
    pub userid: String,
    pub comment: String,
}

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InternalServerError,
}

#[get("/index")]
pub fn index() -> Template {
    info!("Rendering index web");
    let mut context = Context::new();
    Template::render("main", &context)
}

#[get("/share")]
pub fn single_page_app() -> io::Result<NamedFile> {
    info!("Loading static web");
    NamedFile::open("static/build/index.html")
}

#[get("/main")]
pub fn main() -> Template {
    info!("Loading main web");
    let mongo_host = env::var("MONGO_HOST").unwrap_or("localhost".to_string());
    let mongo_port: u16 = env::var("MONGO_PORT")
        .unwrap_or("27017".to_string())
        .parse::<u16>()
        .expect("It is not a number.");
    let news_repo = NewsRepository::new(mongo_host.clone(), mongo_port.clone());

    let mut rt = tokio::runtime::Runtime::new().unwrap();
    //TODO how to manage this error
    // find by user?
    let fut = news_repo.all();
    let news = rt.block_on(fut).unwrap_or(Vec::new());
    let mut context = Context::new();
    context.insert("news", &news);
    let user = User::new("0", "anonymous", "");
    context.insert("user", &user);
    Template::render("main", &context)
}

#[post(
    "/<articleid>/new_comment",
    format = "application/json",
    data = "<comment>"
)]
pub fn new_comment(articleid: String, comment: Json<CommentDTO>) -> status::Accepted<String> {
    info!("Loading add new comment");
    let mongo_host = env::var("MONGO_HOST").unwrap_or("localhost".to_string());
    let mongo_port = env::var("MONGO_PORT")
        .unwrap_or("27017".to_string())
        .parse::<u16>()
        .expect("It is not a number.");
    let comment_repo = CommentRepository::new(mongo_host.clone(), mongo_port.clone());

    // TODO get json data information.
    let _ = comment_repo.insert_one(Comment::new(
        comment.userid.as_str(),
        articleid.as_str(),
        comment.comment.as_str(),
    ));
    status::Accepted(Some("{'result':'ok'}".to_string()))
}
