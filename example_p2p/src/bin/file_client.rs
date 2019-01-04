#[macro_use]
extern crate serde_derive;

extern crate crypto;
extern crate blake2;
extern crate serde;
extern crate serde_json;

//use self::crypto::digest::Digest;
//use self::crypto::sha1::Sha1;
use blake2::{Blake2b, Digest};

use std::io;
use std::io::prelude::*;
use std::fs::File;

use std::collections::HashMap;
//use std::collections::LinkedList;
//use std::fs::{self, DirEntry};
//use std::path::Path;




/*
fn read_dir(dir: &Path) -> LinkedList<String> {
   let result: LinkedList<String> = LinkedList::new();
   if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            //TODO not discard folders
            if !path.is_dir() {
               result.push(path.to_string()); 
            }
        }
    }
   result
}
*/

#[derive(Serialize, Deserialize, Debug)]
pub struct Elem<'a> {
    filename: &'a str,
    sha_id: Vec<u8>,
}

impl<'a> Elem<'a> {
    fn new(filename: &'a str) -> Elem<'a> {
        //TODO add checkers
        let mut fil = File::open(filename).unwrap();
        let mut buffer = Vec::new();
        fil.read_to_end(&mut buffer).unwrap();
        
        let mut hasher = Blake2b::new();
        hasher.input(&mut buffer);
        let sha_id = hasher.result().as_slice().to_vec();
        Elem {filename: filename, sha_id: sha_id}  
    }

    fn to_json(&self) -> String {
        let serialized = serde_json::to_string(&self).unwrap();
        serialized
    }
}


//TODO add for bucket
#[derive(Clone, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Node<'a> {
    identifier: &'a str,
    sha_id: Vec<u8>,
    successor_id: Option<Vec<u8>>,
    predecessor_id: Option<Vec<u8>>,
}


impl<'a> Node<'a> {
    fn new(identifier: &'a str) -> Node<'a> {
        let hex = Node::new_sha1(identifier);
        Node {identifier: identifier, sha_id: hex, successor_id: None, predecessor_id: None}
    }

    fn to_json(&self) -> String {
        let serialized = serde_json::to_string(&self).unwrap();
        serialized
    }
    fn new_sha1(identifier: &'a str)-> Vec<u8> {
        let mut hasher = Blake2b::new();
        hasher.input(identifier);
        hasher.result().as_slice().to_vec()
    }
}

pub struct Application<'a> {
    hash_map: HashMap<&'a str, Node<'a>>,
}

impl<'a> Application<'a> {
    fn join(&mut self, node: Node<'a>) {
        let str_sha = String::from_utf8(node.clone().sha_id).unwrap();
        self.hash_map.insert(str_sha, node);
    }
}



fn main () -> io::Result<()> {
    let address = "127.0.0.1:12345";
    let node = Node::new(address);
    println!("{}", node.to_json()); 
    Ok(())
}
