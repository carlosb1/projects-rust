extern crate termion;

use clap::Parser;

struct DataRepository {}
impl DataRepository {
    pub fn new(file_path: &str) -> Result<DataRepository, &'static str> {
        Ok(DataRepository {})
    }

    pub fn add(&mut self, link: String, tags: Vec<String>) -> Result<(), &'static str> {
        println!("Add!!");
        Ok(())
    }
    pub fn list(&mut self) -> Result<Vec<String>, &'static str> {
        println!("list");
        Ok(Vec::new())
    }

    pub fn remove(&mut self) {
        println!("remove");
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    link: String,
    tags: Vec<String>,
    #[arg(short, long)]
    delete: bool,
    #[arg(short, long)]
    list: bool,
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let opt_args = Args::parse();
    let mut repo = DataRepository::new("data.db")?;
    if opt_args.list {
        let values = repo.list()?;
        values.iter().for_each(|e| println!("- {:}", e));
    } else if opt_args.delete {
        repo.remove();
    } else {
        let _ = repo.add(args.link, args.tags);
    }

    Ok(())
}
