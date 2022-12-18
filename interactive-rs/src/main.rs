extern crate termion;

use clap::Parser;
use sled::Db;

struct DataRepository {
    tree: Db,
}
impl DataRepository {
    pub fn new(file_path: &str) -> Result<DataRepository, String> {
        match sled::open(file_path) {
            Ok(tr) => Ok(DataRepository { tree: tr }),
            Err(e) => Err(format!("{:?}", e)),
        }
    }

    pub fn add(&mut self, link: String, tags: Vec<String>) {
        tags.iter().for_each(|tag| {
            let _ = self.tree.insert(tag.as_str(), link.as_str());
        });

        let _ = self.tree.flush();
    }
    pub fn list(&mut self) {
        //let all_values: Vec<&str> = self.tree.iter().map(|val| val.unwrap().into()).collect();
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
