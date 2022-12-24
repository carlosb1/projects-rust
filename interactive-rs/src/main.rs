extern crate termion;

use clap::Parser;
use redis;
use redis::Commands;

struct DataRepository {
    client: redis::Client,
}
impl DataRepository {
    pub fn new(file_path: &str) -> Result<DataRepository, &'static str> {
        Ok(DataRepository {
            client: redis::Client::open(file_path)
                .map_err(|e| "It was not possible to open the file")?,
        })
    }

    pub fn add(&mut self, link: String, tags: Vec<String>) -> Result<(), &'static str> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| "It could not get a connection")?;

        tags.iter().for_each(|tag| {
            let _: () = con
                .set(tag.as_str(), link.as_str())
                .expect("It could not set the values in the connection");
        });

        Ok(())
    }
    pub fn list(&mut self) -> Result<(), &'static str> {
        let mut con = self
            .client
            .get_connection()
            .map_err(|e| "It could not get a connection")?;

        let _: () = redis::cmd("KEYS").arg("*").query(&mut con).unwrap();

        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    link: String,
    tags: Vec<String>,
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    delete: bool,
    #[arg(short, long)]
    list: bool,
}

fn main() -> Result<(), String> {
    let args = Cli::parse();
    let mut repo = DataRepository::new("data.db")?;
    repo.add(args.link, args.tags);
    Ok(())
}
