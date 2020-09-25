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

pub fn load_mongo_credentials() -> (String, u16) {
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
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run() -> Context {
        let (mongo_host, mongo_port) = load_mongo_credentials();
        let news_repo = NewsRepository::new(mongo_host.clone(), mongo_port.clone());
        let comment_repo = CommentRepository::new(mongo_host.clone(), mongo_port.clone());

        //TODO how to filter news
        // find by user? or more popular news.
        let values = news_repo.all().await;
        let news = values.unwrap_or(Vec::new());
        let id_news: Vec<String> = news
            .clone()
            .into_iter()
            .map(|new| new.id.to_string())
            .rev()
            .collect();
        let mut value_comments: Vec<Vec<Vec<String>>> = Vec::new();
        for id_new in id_news.clone() {
            //vec of comment
            let comments_by_id = comment_repo
                .clone()
                .find_by_new_id(&id_new)
                .await
                .unwrap_or(Vec::new())
                .into_iter()
                .map(|comment| vec![comment.iduser, comment.idnew, comment.comment])
                .rev()
                .collect();
            value_comments.push(comments_by_id);
        }

        let comments: HashMap<_, _> = id_news
            .clone()
            .into_iter()
            .zip(value_comments.into_iter())
            .collect();
        let mut context = Context::new();
        context.insert("news", &news);
        context.insert("comments", &comments);
        //TODO check this comment.
        let user = User::new("0", "anonymous", "");
        context.insert("user", &user);
        context
    }
    let context = rt.block_on(run());
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

        if let Some(mut user) = user_repo.clone().find_one(userid.as_str()).await {
            if !user.fake_articles.contains(&articleid.clone()) {
                user.fake_articles.push(articleid.clone());
                user_repo.clone().update(user).await;
                if let Some(mut new) = news_repo.clone().find_one(articleid.as_str()).await {
                    new.fake += 1;
                    news_repo.update(new).await;
                }
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
        if let Some(mut user) = user_repo.clone().find_one(userid.as_str()).await {
            if !user.like_articles.contains(&articleid.clone()) {
                user.like_articles.push(articleid.clone());
                user_repo.clone().update(user).await;
                if let Some(mut new) = news_repo.clone().find_one(articleid.as_str()).await {
                    new.liked += 1;
                    news_repo.update(new).await;
                }
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

        if let Some(mut user) = user_repo.clone().find_one(userid.as_str()).await {
            if !user.approved_articles.contains(&articleid.clone()) {
                user.approved_articles.push(articleid.clone());
                user_repo.clone().update(user).await;
                if let Some(mut new) = news_repo.clone().find_one(articleid.as_str()).await {
                    new.approved += 1;
                    news_repo.update(new).await;
                }
            }
        }
    }
    rt.block_on(run(articleid, user_id.userid.clone()));
    status::Accepted(Some("{'result':'ok'}".to_string()))
}
