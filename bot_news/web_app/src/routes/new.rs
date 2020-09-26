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
use crypto::digest::Digest;
use crypto::sha3::Sha3;
use rocket::http::{Cookie, Cookies};
use serde::{Deserialize, Serialize};
//TODO change name endpoint

#[derive(Deserialize, Clone)]
pub struct CommentDTO {
    pub userid: String,
    pub comment: String,
}
#[derive(Deserialize, Clone)]
pub struct TagDTO {
    pub userid: String,
    pub tags: Vec<String>,
}
#[derive(Deserialize, Clone)]
pub struct SearchDTO {
    pub ids: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct UserIdDTO {
    pub userid: String,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct LoginDTO {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    InternalServerError,
}

pub fn hash_password(password: &String) -> String {
    let mut hasher = Sha3::sha3_256();
    hasher.input_str(password);
    hasher.result_str()
}

#[post("/login", format = "json", data = "<login_info>")]
pub fn login(login_info: Json<LoginDTO>, mut cookies: Cookies) -> Json<Option<String>> {
    info!("Applying logging");
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run(name: &str, password: &str) -> Option<String> {
        //TODO check this comment
        let (mongo_host, mongo_port) = load_mongo_credentials();
        let user_repo = UserRepository::new(mongo_host.clone(), mongo_port.clone());
        if let Some(user) = user_repo.find_one_by_name(name).await {
            if (user.password == hash_password(&password.to_string())) {
                Some(user.id.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    let result = rt.block_on(run(
        login_info.username.as_str(),
        login_info.password.as_str(),
    ));
    if (result.is_some()) {
        let cookie = Cookie::build("userid", result.clone().unwrap())
            .secure(true)
            .finish();
        cookies.add(cookie);
    }
    Json(result)
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

#[post("/search", format = "application/json", data = "<searchIds>")]
pub fn search(searchIds: Json<SearchDTO>) -> Template {
    info!("Loading main web");
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run(userid: &str, ids: Vec<String>) -> Context {
        //TODO check this comment.
        let (mongo_host, mongo_port) = load_mongo_credentials();

        let user_repo = UserRepository::new(mongo_host.clone(), mongo_port.clone());
        let news_repo = NewsRepository::new(mongo_host.clone(), mongo_port.clone());
        let comment_repo = CommentRepository::new(mongo_host.clone(), mongo_port.clone());
        let user = user_repo
            .clone()
            .find_one(userid)
            .await
            .unwrap_or(User::new("0", "anonymous", ""));
        //TODO how to filter news
        // find by user? or more popular news.
        let values = news_repo.find_by_ids(ids).await;
        let news = values.unwrap_or(Vec::new());
        let id_news: Vec<String> = news
            .clone()
            .into_iter()
            .map(|new| new.id.to_string())
            .rev()
            .collect();
        let mut value_comments: Vec<Vec<Vec<String>>> = Vec::new();
        let mut info: HashMap<_, _> = HashMap::new();
        let mut tags: HashMap<_, _> = HashMap::new();
        for new in news.clone() {
            //vec of comment
            let comments_by_id = comment_repo
                .clone()
                .find_by_new_id(&new.id)
                .await
                .unwrap_or(Vec::new())
                .into_iter()
                .map(|comment| vec![comment.iduser, comment.idnew, comment.comment])
                .rev()
                .collect();
            value_comments.push(comments_by_id);
            let mut values_info: HashMap<String, bool> = HashMap::new();
            values_info.insert("star".to_string(), user.like_articles.contains(&new.id));
            values_info.insert("fake".to_string(), user.fake_articles.contains(&new.id));
            values_info.insert(
                "approve".to_string(),
                user.approved_articles.contains(&new.id),
            );
            info.insert(new.id.clone(), values_info);
            tags.insert(new.id.clone(), new.tags.join(","));
        }

        let comments: HashMap<_, _> = id_news
            .clone()
            .into_iter()
            .zip(value_comments.into_iter())
            .collect();

        //fake data
        let mut context = Context::new();
        context.insert("news", &news);
        context.insert("comments", &comments);
        context.insert("user", &user);
        context.insert("info", &info);
        context.insert("tags", &tags);
        context
    }
    let context = rt.block_on(run("0", searchIds.ids.clone()));
    Template::render("main", &context)
}

#[get("/main")]
pub fn main() -> Template {
    info!("Loading main web");
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run(userid: &str) -> Context {
        //TODO check this comment.
        let (mongo_host, mongo_port) = load_mongo_credentials();

        let user_repo = UserRepository::new(mongo_host.clone(), mongo_port.clone());
        let news_repo = NewsRepository::new(mongo_host.clone(), mongo_port.clone());
        let comment_repo = CommentRepository::new(mongo_host.clone(), mongo_port.clone());
        let user = user_repo
            .clone()
            .find_one(userid)
            .await
            .unwrap_or(User::new("0", "anonymous", ""));
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
        let mut info: HashMap<_, _> = HashMap::new();
        let mut tags: HashMap<_, _> = HashMap::new();
        for new in news.clone() {
            //vec of comment
            let comments_by_id = comment_repo
                .clone()
                .find_by_new_id(&new.id)
                .await
                .unwrap_or(Vec::new())
                .into_iter()
                .map(|comment| vec![comment.iduser, comment.idnew, comment.comment])
                .rev()
                .collect();
            value_comments.push(comments_by_id);
            let mut values_info: HashMap<String, bool> = HashMap::new();
            values_info.insert("star".to_string(), user.like_articles.contains(&new.id));
            values_info.insert("fake".to_string(), user.fake_articles.contains(&new.id));
            values_info.insert(
                "approve".to_string(),
                user.approved_articles.contains(&new.id),
            );
            info.insert(new.id.clone(), values_info);
            tags.insert(new.id.clone(), new.tags.join(","));
        }

        let comments: HashMap<_, _> = id_news
            .clone()
            .into_iter()
            .zip(value_comments.into_iter())
            .collect();

        //fake data
        let mut context = Context::new();
        context.insert("news", &news);
        context.insert("comments", &comments);
        context.insert("user", &user);
        context.insert("info", &info);
        context.insert("tags", &tags);
        context
    }
    let context = rt.block_on(run("0"));
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

#[post("/<articleid>/save_tags", format = "application/json", data = "<tags>")]
pub fn save_tags(articleid: String, tags: Json<TagDTO>) -> status::Accepted<String> {
    info!(
        "{articleid}, {userid}",
        articleid = articleid.as_str(),
        userid = tags.clone().userid.as_str(),
    );
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    async fn run(articleid: String, tags: Vec<String>, userid: &str) {
        let (mongo_host, mongo_port) = load_mongo_credentials();
        let user_repo = UserRepository::new(mongo_host.clone(), mongo_port.clone());
        let news_repo = NewsRepository::new(mongo_host.clone(), mongo_port.clone());

        if let Some(mut user) = user_repo.clone().find_one(userid).await {
            if let Some(mut new) = news_repo.clone().find_one(articleid.as_str()).await {
                new.tags = tags.clone();
                info!("trying the update {:?}", tags.clone());
                news_repo.update(new).await;
            }
        }
    }
    rt.block_on(run(articleid, tags.tags.clone(), tags.userid.as_str()));
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
            if !user.fake_articles.contains(&articleid) {
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
            if !user.like_articles.contains(&articleid) {
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
            if !user.approved_articles.contains(&articleid) {
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
