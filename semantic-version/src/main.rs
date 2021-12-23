use clap::Parser;
use colored::*;
use std::path::{Path, PathBuf};
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

#[derive(Debug)]
pub struct FoundEntry {
    path: PathBuf,
    pos: usize,
}

fn check_dir(path: &str, query: &str) -> Vec<FoundEntry> {
    let mut total_files_scanned = 0;

    let mut entries = Vec::new();
    for (fl_no, e) in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .enumerate()
    {
        if e.metadata().unwrap().is_file() {
            match fstream::contains(e.path(), query) {
                Some(b) => {
                    if b {
                        entries.append(&mut check_file(e.path(), query));
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
    entries
}

fn check_file(file_path: &Path, query: &str) -> Vec<FoundEntry> {
    println!(
        "In file {}\n",
        file_path.display().to_string().magenta().italic()
    );
    let mut entries = Vec::new();
    match fstream::read_lines(file_path) {
        Some(s) => {
            for (pos, s) in s.iter().enumerate() {
                if s.contains(query) {
                    print!("{}", "Line ".green().bold());
                    print!("{0: <6} ", pos.to_string().cyan());
                    println!("=> {}", s.trim().blue());
                    let found_entry = FoundEntry {
                        path: PathBuf::from(file_path),
                        pos,
                    };
                    entries.push(found_entry);
                }
            }
        }
        None => println!("Error in reading File"),
    }

    println!("");
    entries
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
    let found_entries = check_dir(&path, &query);

    for entry in found_entries.iter() {
        println! {"{:?}", entry}
    }
}
