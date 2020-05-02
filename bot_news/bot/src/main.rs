use scraper::{Html, Selector};
use teloxide::prelude::*;
use url::Url;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hasher};

const ADDRESS_SERVER_SEARCHER: &str = "http://127.0.0.1:7700/indexes/news/documents";
const ADDRESS_SERVER_ML: &str = "http://127.0.0.1:5002/api/news";

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
                        match download_and_parse(text.to_string()).await{
                            Ok((link, title, description)) => {
                                    upload_link_info(link.as_str(), title.as_str(), description.as_str()).await;
                            }
                            Err(str_e) => {log::error!(" {:}",&str_e);}
                        };
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

async fn download_and_parse<'a> (link: String) -> Result<(String, String, String), &'a str>{
    match reqwest::get(link.as_str()).await{
        Ok(info) => {
            let body = info.text().await.unwrap_or("fuck you".to_string());
            let fragment = Html::parse_document(&body);
            let title = fragment.select(&Selector::parse(r#"meta[property="og:title"]"#).unwrap()).next().unwrap().value().attr("content").unwrap_or("");
            let description = fragment.select(&Selector::parse(r#"meta[property="og:description"]"#).unwrap()).next().unwrap().value().attr("content").unwrap_or("");
            log::info!(" title {:?}", title);
            log::info!(" descrp {:?}", description);
            Ok((link, title.to_string(), description.to_string()))
        }
        Err(_) => { 
            log::error!("I screwed up");
            Err("It was not possible to download")
        }
    }

}


async fn upload_link_info (link: &str, title: &str, descrip: &str) {
    log::info!("Uploading link info");
    let mut info_to_upload = HashMap::new();
    let mut hasher = DefaultHasher::new();
    hasher.write(link.as_bytes());
    let id = hasher.finish().to_string();
    info_to_upload.insert("id", id.as_str());
    info_to_upload.insert("link", link);
    info_to_upload.insert("title", title);
    info_to_upload.insert("description", descrip);
    
    let list_uploads = vec!{info_to_upload};
    // Upload info in search server
    let res = reqwest::Client::new().post(ADDRESS_SERVER_SEARCHER).json(&list_uploads).send().await;
    match res {
        Ok(_) =>  {log::info!("It was uploaded correctly in the search server");}
        Err(e) => {log::info!("{:}", e);}
    };
    // Upload info in ml server
    let res = reqwest::Client::new().post(ADDRESS_SERVER_ML).json(&list_uploads).send().await;
    match res {
        Ok(_) =>  {log::info!("It was uploaded correctly in the ml server");}
        Err(e) => {log::info!("{:}", e);}
    };
}


