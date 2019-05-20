extern crate reqwest;
extern crate select;
extern crate dynomite;
extern crate rusoto_core;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use http::StatusCode;
use lambda_http::{lambda, Body, IntoResponse, Request, Response};
use lambda_runtime::{error::HandlerError, Context};
use std::error::Error;

mod db;

use select::document::Document;
use select::predicate::{Class, Name, Predicate};


static NAME_ID: &'static str  = "id";
static NAME_TABLE: &'static str  = "test";

#[derive(Deserialize, Serialize, Debug)]
struct DTONews {
    link: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    lambda!(router);
    Ok(())
}


fn router(req: Request, c: Context) -> Result<impl IntoResponse, HandlerError> {
    let client = db::DummyClientDB::new(NAME_ID.to_string(), NAME_TABLE.to_string());
    match req.method().as_str() {
        "GET" => get_news(req, c, client),
        "POST" => add_news(req, c, client),
        _ => not_allowed(req, c),
    }
}

fn not_allowed(_req: Request, _c: Context) -> Result<Response<Body>, HandlerError> {
    Ok(Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::from(()))
        .expect("err creating response"))
}

fn get_news(_req: Request, _c: Context, client: db::DummyClientDB) -> Result<Response<Body>, HandlerError> {
    Ok(serde_json::json!(client.clone().list()).into_response())
}

fn scrap_news(web_news: &str) -> Result<(), Box<std::error::Error>>  {
    let resp = reqwest::get(web_news)?.text()?;
    let document = Document::from(resp.as_str());
    let to_parse_text = document.find(Class("content-structure")).next().unwrap().text();
    Ok(())
}

fn add_news(req: Request, _c: Context, client: db::DummyClientDB) -> Result<Response<Body>, HandlerError> {
    //let web_news = "https://www.lavanguardia.com/internacional/20190518/462299498579/iran-eeuu-armada-china-golfo-persico.html";
    match serde_json::from_slice::<DTONews>(req.body().as_ref()) {
        Ok(news) => {
           //TODO add asyncron
            scrap_news(news.link.as_str());
            let mut resp = serde_json::json!(news).into_response();
            *resp.status_mut() = StatusCode::CREATED;
            Ok(resp)
        }
        Err(_) => Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("bad request".into())
            .expect("err creating response")),
    }
}


