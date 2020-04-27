extern crate select;
extern crate reqwest;
extern crate scraper;

use scraper::{Html, Selector};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
    let res = reqwest::blocking::get("https://elpais.com/tecnologia/2020-04-27/whatsapp-asegura-que-la-medida-para-limitar-el-reenvio-de-mensajes-ha-reducido-la-viralidad-en-un-70.html")?;
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
