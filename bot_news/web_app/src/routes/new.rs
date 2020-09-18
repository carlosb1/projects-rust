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

#[derive(Deserialize, Clone)]
pub struct UserIdDTO {
    pub userid: String,
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

fn load_mongo_credentials() -> (String, u16) {
    let mongo_host = env::var("MONGO_HOST").unwrap_or("localhost".to_string());
    let mongo_port: u16 = env::var("MONGO_PORT")
        .unwrap_or("27017".to_string())
        .parse::<u16>()
        .expect("It is not a number.");
    (mongo_host, mongo_port)
}

#[get("/main")]
pub fn main() -> Template {
    info!("Loading main web");
    let (mongo_host, mongo_port) = load_mongo_credentials();
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
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run(articleid: String, userid: String, comment: String) {
        let (mongo_host, mongo_port) = load_mongo_credentials();
        let comment_repo = CommentRepository::new(mongo_host.clone(), mongo_port.clone());
        let _ = comment_repo
            .insert_one(Comment::new(
                userid.as_str(),
                articleid.as_str(),
                comment.as_str(),
            ))
            .await;
    }
    rt.block_on(run(
        articleid,
        comment.userid.clone(),
        comment.comment.clone(),
    ));
    status::Accepted(Some("{'result':'ok'}".to_string()))
}

#[post("/<articleid>/fake", format = "application/json", data = "<user_id>")]
pub fn fake(articleid: String, user_id: Json<UserIdDTO>) -> status::Accepted<String> {
    info!("New fake");
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run(articleid: String, userid: String) {
        let (mongo_host, mongo_port) = load_mongo_credentials();
        let user_repo = UserRepository::new(mongo_host.clone(), mongo_port.clone());
        let news_repo = NewsRepository::new(mongo_host.clone(), mongo_port.clone());

        if let Some(mut user) = user_repo.find_one(userid.as_str()).await {
            user.fake_articles.push(articleid.clone());
            if let Some(mut new) = news_repo.clone().find_one(articleid.as_str()).await {
                new.fake += 1;
                news_repo.insert_one(new).await;
            }
        }
    }
    rt.block_on(run(articleid, user_id.userid.clone()));
    status::Accepted(Some("{'result':'ok'}".to_string()))
}

#[post("/<articleid>/like", format = "application/json", data = "<user_id>")]
pub fn like(articleid: String, user_id: Json<UserIdDTO>) -> status::Accepted<String> {
    info!("New like");
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run(articleid: String, userid: String) {
        let (mongo_host, mongo_port) = load_mongo_credentials();
        let user_repo = UserRepository::new(mongo_host.clone(), mongo_port.clone());
        let news_repo = NewsRepository::new(mongo_host.clone(), mongo_port.clone());
        if let Some(mut user) = user_repo.find_one(userid.as_str()).await {
            user.fake_articles.push(articleid.clone());
            if let Some(mut new) = news_repo.clone().find_one(articleid.as_str()).await {
                new.liked += 1;
                news_repo.insert_one(new).await;
            }
        }
    }
    rt.block_on(run(articleid, user_id.userid.clone()));

    status::Accepted(Some("{'result':'ok'}".to_string()))
}

#[post(
    "/<articleid>/approve",
    format = "application/json",
    data = "<user_id>"
)]
pub fn approve(articleid: String, user_id: Json<UserIdDTO>) -> status::Accepted<String> {
    info!("New approve");
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run(articleid: String, userid: String) {
        let (mongo_host, mongo_port) = load_mongo_credentials();
        let user_repo = UserRepository::new(mongo_host.clone(), mongo_port.clone());
        let news_repo = NewsRepository::new(mongo_host.clone(), mongo_port.clone());

        if let Some(mut user) = user_repo.find_one(userid.as_str()).await {
            user.approved_articles.push(articleid.clone());
            if let Some(mut new) = news_repo.clone().find_one(articleid.as_str()).await {
                new.approved += 1;
                news_repo.insert_one(new);
            }
        }
    }
    rt.block_on(run(articleid, user_id.userid.clone()));
    status::Accepted(Some("{'result':'ok'}".to_string()))
}
