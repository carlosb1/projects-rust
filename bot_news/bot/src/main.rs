use scraper::{Html, Selector};
use teloxide::prelude::*;
use url::Url;
use std::collections::HashMap;

const ADDRESS_SERVER_NEWS: &str = "http://127.0.0.1:7700/";

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
                        log::info!("We got a link! ");
                        let _ = download_and_parse(text).await;    
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

async fn download_and_parse<'a> (link: &str) -> Result<(), &'a str>{
    match reqwest::get(link).await{
        Ok(info) => {
            let body = info.text().await.unwrap_or("fuck you".to_string());
            let fragment = Html::parse_document(&body);
            let title = fragment.select(&Selector::parse(r#"meta[property="og:title"]"#).unwrap()).next().unwrap().value().attr("content");
            let description = fragment.select(&Selector::parse(r#"meta[property="og:description"]"#).unwrap()).next().unwrap().value().attr("content");
            log::info!(" title {:?}", title.unwrap());
            log::info!(" descrp {:?}", description.unwrap());
            Ok(())
        }
        Err(_) => { 
            log::error!("I screwed up");
            Err("It was not possible to download")
        }
    }

}


async fn upload_link_info (link: &str, title: &str, descrip: &str) {
    let mut info_to_upload = HashMap::new();
    info_to_upload.insert("link", link);
    info_to_upload.insert("title", title);
    info_to_upload.insert("description", descrip);
    
    let res = reqwest::Client::new().post(ADDRESS_SERVER_NEWS).json(&info_to_upload).send().await;
    match res {
        Ok(_) =>  {log::info!("It was uploaded correctly");}
        Err(e) => {log::info!("{:}", e);}
    };
}


