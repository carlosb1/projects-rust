use clap::Parser;
use colored::*;
use glob::glob;
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
    s: String,
}
fn check_dirs(path: &str, query: &str) -> Vec<FoundEntry> {
    let mut entries = Vec::new();
    for entry in glob(path).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                println!("!!!!!!!!!!!!!{:?}", path);
                entries.append(&mut check_file(path, query));
            }
            Err(e) => println!("{:?}", e),
        }
    }
    entries
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
                        entries.append(&mut check_file(PathBuf::from(e.path()), query));
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

fn check_file(path: PathBuf, query: &str) -> Vec<FoundEntry> {
    println!(
        "In file {}\n",
        path.display().to_string().magenta().italic()
    );
    let mut entries = Vec::new();
    match fstream::read_lines(path.clone()) {
        Some(s) => {
            for (pos, s) in s.iter().enumerate() {
                if s.contains(query) {
                    let found_entry = FoundEntry {
                        path: path.clone(),
                        pos,
                        s: s.clone(),
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
    let first_entries = check_dirs(&path, &query);
    first_entries.iter().for_each(|entry| {
        print!("1/ {}", "Line ".green().bold());
        print!("1/ {0: <6} ", entry.pos.to_string().cyan());
        println!("1/ => {}", entry.s.trim().blue());
    });

    let found_entries = check_dir(&path, &query);
    found_entries.iter().for_each(|entry| {
        print!("{}", "Line ".green().bold());
        print!("{0: <6} ", entry.pos.to_string().cyan());
        println!("=> {}", entry.s.trim().blue());
    });
}
