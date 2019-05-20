#[macro_use]
extern crate lambda_runtime as lambda;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate simple_logger;


extern crate reqwest;
extern crate select;
extern crate dynomite;
extern crate rusoto_core;

mod db;

use select::document::Document;
use select::predicate::{Class, Name, Predicate};
use lambda::error::HandlerError;

use std::error::Error;




static NAME_ID: &'static str  = "id";
static NAME_TABLE: &'static str  = "test";



#[derive(Deserialize, Clone)]
struct CustomEvent {
}

#[derive(Serialize, Clone)]
struct CustomOutput {
    message: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Info)?;
    lambda!(my_handler);
    Ok(())
}

fn my_handler(e: CustomEvent, c: lambda::Context) -> Result<CustomOutput, HandlerError> {
    let client = db::DummyClientDB::new(NAME_ID.to_string(), NAME_TABLE.to_string());
    check_news("https://news.ycombinator.com", client);
    Ok(CustomOutput {
        message: format!("inserted"),
    })
}


fn check_news(url: &str, client: db::DummyClientDB) {
    let resp = reqwest::get(url).unwrap();
    assert!(resp.status().is_success());
    let document = Document::from_read(resp).unwrap();
    
    for node in document.find(Class("athing")) {
        let rank = node.find(Class("rank")).next().unwrap();
        let story = node.find(Class("title").descendant(Name("a")))
            .next()
            .unwrap()
            .text();
        let url = node.find(Class("title").descendant(Name("a")))
            .next()
            .unwrap();
        let url_txt = url.attr("href").unwrap();
        //let url_trim = url_txt.trim_left_matches('/');
        println!("rank {} story {} url {}", rank.text(), story, url_txt);
        let news = db::News::new(url_txt.to_string(), story.to_string());
        client.clone().put(&news);

    }
}
