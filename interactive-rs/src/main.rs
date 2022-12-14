extern crate termion;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    link: String,
    tags: Vec<String>,
}

fn main() {
    let args = Cli::parse();

    println!("{:?}", args.link);
    println!("{:?}", args.tags);
}
