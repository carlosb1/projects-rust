extern crate crypto; 

use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

use std::io;
use std::io::prelude::*;
use std::fs::File;

pub struct Node<'a> {
    filename: &'a str,
    sha_id: String,
    raw_info: Vec<u8>,
}

impl<'a> Node<'a> {
    fn new(filename: &str) -> Node {
        //TODO add checkers
        let mut fil = File::open(filename).unwrap();
        let mut buffer = Vec::new();
        fil.read_to_end(&mut buffer).unwrap();
        let mut hasher = Sha1::new();
        hasher.input(&mut buffer);
        let hex = hasher.result_str();
        // calculate sha1
        Node {filename: filename, sha_id: hex, raw_info: buffer}
    }
}

fn main () -> io::Result<()> {
    let str_value = "/home/carlosb/Desktop/2018-12-21.png".to_string(); 
    Node::new(&str_value);

    Ok(())
}
