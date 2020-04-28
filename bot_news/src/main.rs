use serde::{Deserialize, Serialize};
use scraper::{Html, Selector};
use teloxide::prelude::*;


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
            let text = message.update.text().unwrap();
            download_and_parse(text);
            message.answer("pong").send().await.log_on_error().await;
       })
    }).dispatch().await;

}

fn download_and_parse (text: &str) {
    let res = reqwest::blocking::get("https://elpais.com/tecnologia/2020-04-27/whatsapp-asegura-que-la-medida-para-limitar-el-reenvio-de-mensajes-ha-reducido-la-viralidad-en-un-70.html").unwrap();
    println!("Status = {}", res.status());
    println!("Headers = {:?}", res.headers());
    
    let body = res.text().unwrap();

    let fragment = Html::parse_document(&body);
    let title = fragment.select(&Selector::parse(r#"meta[property="og:title"]"#).unwrap()).next().unwrap().value().attr("content");
    let description = fragment.select(&Selector::parse(r#"meta[property="og:description"]"#).unwrap()).next().unwrap().value().attr("content");
    println!("{:?}", title.unwrap());
    println!("{:?}", description.unwrap());

}


    /*
fn main() -> Result<(), Box<dyn std::error::Error>> {

    Ok(())
}
    */
