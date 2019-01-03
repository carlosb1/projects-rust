#[macro_use]
extern crate serde_derive;

extern crate crypto; 
extern crate serde;
extern crate serde_json;

use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

use std::io;
use std::io::prelude::*;
use std::fs::File;


#[derive(Serialize, Deserialize, Debug)]
pub struct Node<'a> {
    filename: &'a str,
    address: &'a str,
    sha_id: String,
    //raw_info: Vec<u8>,
}

impl<'a> Node<'a> {
    fn new(filename: &'a str, address: &'a str) -> Node<'a> {
        //TODO add checkers
        let mut fil = File::open(filename).unwrap();
        let mut buffer = Vec::new();
        fil.read_to_end(&mut buffer).unwrap();
        let mut hasher = Sha1::new();
        hasher.input(&mut buffer);
        let hex = hasher.result_str();
        // calculate sha1
        Node {filename: filename, address: address, sha_id: hex}
    }

    fn to_json(&self) -> String {
        let serialized = serde_json::to_string(&self).unwrap();
        serialized
    }
}

fn main () -> io::Result<()> {
    let str_value = "/home/carlosb/Desktop/2018-12-21.png";
    let address = "127.0.0.1:12345";
    let node = Node::new(str_value, address);
    println!("{}", node.to_json()); 
    Ok(())
}
