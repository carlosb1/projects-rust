extern crate termion;

use clap::Parser;

struct DataRepository {
    file_path: String,
}
impl DataRepository {
    pub fn add(self, link: String, tags: Vec<String>) {
        let tree = sled::open(self.file_path.to_string())
            .map_err(|x| format!("Error: {x}").to_string())
            .expect(format!("It was not possible open the file {:?}", self.file_path).as_str());
        tags.iter().for_each(|tag| {
            let _ = tree.insert(tag.as_str(), link.as_str());
        });
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    link: String,
    tags: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    let repo = DataRepository {
        file_path: "data.db".to_string(),
    };
    repo.add(args.link, args.tags);
}
