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
    sha_id: u32,
    successor_id: Option<u32>,
    predecessor_id: Option<u32>,
}


impl<'a> Node<'a> {
    fn new(identifier: &'a str) -> Node<'a> {
        let sha_id = to_u32(&Node::new_sha1(identifier));
        Node {identifier: identifier, sha_id: sha_id, successor_id: None, predecessor_id: None}
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

fn to_u32(vec: & Vec<u8>) -> u32 { 
   vec.iter().rev().fold(0, |acc, &b| acc*2 + b as u32) 
}

fn to_array(x: u32) -> [u8; 4] {
    let b1: u8 = ((x >> 24) & 0xff) as u8;
    let b2: u8 = ((x >> 16) & 0xff) as u8;
    let b3: u8 = ((x >> 8) & 0xff) as u8;
    let b4: u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}



pub struct Application<'a> {
    hash_map: HashMap<u32, Node<'a>>,
    first_node: Option<u32>,
}

impl<'a> Application<'a> {
    fn new () -> Application<'a> {
        Application{hash_map: HashMap::new(), first_node: None}
    }
    fn join(&mut self, mut node: Node<'a>) {
        self.first_node = Some(node.sha_id);   
        
        let found_id = self.search(node.sha_id);
   
        if found_id == self.first_node {
            self.first_node = found_id;
        }
        //TODO check if it has predecessor
        let found_node = self.hash_map.get(&found_id.unwrap()); 
        let old_predecessor_id = found_node.unwrap().predecessor_id.unwrap();
        let mut old_predecessor_node  = self.hash_map.get_mut(&old_predecessor_id).unwrap();

        (*old_predecessor_node).successor_id = Some(node.sha_id);
        node.predecessor_id = Some(old_predecessor_node.sha_id);
        node.successor_id = Some(found_id.unwrap());
        //TODO update odes
        self.hash_map.insert(node.sha_id, node);
    }
    
    fn search(&mut self, id_to_search: u32) -> Option<u32> {
        let mut node_to_compare = self.first_node.unwrap();
        while id_to_search  < node_to_compare && id_to_search != node_to_compare {
            let next_node = self.hash_map.get(&node_to_compare).unwrap();
            node_to_compare = next_node.successor_id.unwrap(); 
        }
        Some(id_to_search)
    } 
}



fn main () -> io::Result<()> {
    let node = Node::new("127.0.0.1:12345");
    let node2 = Node::new("127.0.0.1:12346");
    let mut app = Application::new();
    app.join(node);
    app.join(node2);
    
    Ok(())
}
