use clap::Parser;
use colored::*;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    /// Path string
    #[clap(short, long)]
    path: String,

    /// Query string
    #[clap(short, long)]
    query: String,
}

fn check_dir(path: &str, query: &str) {
    let mut total_files_scanned = 0;
    for (fl_no, e) in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .enumerate()
    {
        if e.metadata().unwrap().is_file() {
            match fstream::contains(e.path(), query) {
                Some(b) => {
                    if b {
                        check_file(e.path(), query);
                    }
                }
                None => println!("Error in walking Dir"),
            }
        }
        total_files_scanned = fl_no;
    }

    println!(
        "Total Scanned files {}",
        total_files_scanned.to_string().bold()
    );
}

fn check_file(file_path: &Path, query: &str) {
    println!(
        "In file {}\n",
        file_path.display().to_string().magenta().italic()
    );
    match fstream::read_lines(file_path) {
        Some(s) => {
            for (pos, s) in s.iter().enumerate() {
                if s.contains(query) {
                    print!("{}", "Line ".green().bold());
                    print!("{0: <6} ", pos.to_string().cyan());
                    println!("=> {}", s.trim().blue());
                }
            }
        }
        None => println!("Error in reading File"),
    }
    println!("");
}

fn main() {
    let cli = Args::parse();
    let path = cli.path;
    let query = cli.query;
    println!(
        "Searching '{}' in {}\n",
        query.green().bold(),
        path.italic()
    );
    check_dir(&path, &query);
}
