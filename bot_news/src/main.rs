use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use teloxide::prelude::*;
use url::Url;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewInfo {
    title: String,
    link: String,
    description: String,
}

#[tokio::main]
async fn main(){
    teloxide::enable_logging!();
    log::info!("Starting ping pong bot!");

    let bot = Bot::from_env();

    Dispatcher::new(bot).messages_handler( |rx: DispatcherHandlerRx<Message>| {
       rx.for_each(|message| async move {
            match message.update.text() {
                Some(text) => {
                    if is_link(text) { 
                        download_and_parse(text);    
                    }
                },
                None => {},
            };

            message.answer("pong").send().await.log_on_error().await;
       })
    }).dispatch().await;

}

fn is_link(text: &str) -> bool{
    let possible_link = Url::parse(text);
    match possible_link {
        Ok(_) => true,
        Err(_) => false, 
     }
}

fn download_and_parse<'a> (link: &str) -> Result<(), &'a str>{
    let res = reqwest::blocking::get(link).unwrap();
    println!("Status = {}", res.status());
    println!("Headers = {:?}", res.headers());
    
    let body = res.text().unwrap();

    let fragment = Html::parse_document(&body);
    let title = fragment.select(&Selector::parse(r#"meta[property="og:title"]"#).unwrap()).next().unwrap().value().attr("content");
    let description = fragment.select(&Selector::parse(r#"meta[property="og:description"]"#).unwrap()).next().unwrap().value().attr("content");
    println!("{:?}", title.unwrap());
    println!("{:?}", description.unwrap());
    Ok(())
}


    /*
fn main() -> Result<(), Box<dyn std::error::Error>> {

    Ok(())
}
    */
