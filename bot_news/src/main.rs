extern crate select;
extern crate reqwest;
use select::document::Document;
use select::predicate::{Predicate, Attr, Class, Name};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut res = reqwest::blocking::get("https://www.elpais.com")?;
    println!("Status = {}", res.status());
    println!("Headers = {:?}", res.headers());
    let document = Document::from(res.text()?.as_str());
    Ok(())
}
