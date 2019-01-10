#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;


use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::io;
use std::io::prelude::*;
use std::fs::File;

use std::collections::HashMap;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
}

//TODO add for bucket
#[derive(Clone, Hash, Eq, PartialEq)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    identifier: String,
    sha_id: u64,
    successor_id: Option<u64>,
    predecessor_id: Option<u64>,
}


impl Node {
    fn new(identifier: String) -> Node {
        let sha_id = calculate_hash(&identifier);
        Node {identifier: identifier, sha_id: sha_id, successor_id: None, predecessor_id: None}
    }

    fn to_json(&self) -> String {
        let serialized = serde_json::to_string(&self).unwrap();
        serialized
    }

}

pub struct Application {
    hash_map: HashMap<u64, Node>,
    first_node: Option<u64>,
}

impl Application {
    fn new () -> Application {
        Application{hash_map: HashMap::new(), first_node: None}
    }
    fn join(&mut self, mut node: Node) {
        match &self.first_node {
            None =>  { self.first_node = Some(node.sha_id); 
                node.predecessor_id = Some(node.sha_id);
                node.successor_id = Some(node.sha_id);
                self.hash_map.insert(node.sha_id, node); 
                return },
            _ => println!("Checked first node"),
        }
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
    
    fn search(&mut self, id_to_search: u64) -> Option<u64> {
        let mut node_to_compare = self.first_node.unwrap();
        while id_to_search  < node_to_compare && id_to_search != node_to_compare {
            let next_node = self.hash_map.get(&node_to_compare).unwrap();
            node_to_compare = next_node.successor_id.unwrap(); 
        }
        Some(id_to_search)
    } 
}



fn main () -> io::Result<()> {
    let node = Node::new("127.0.0.1:12345".to_string());
    let node2 = Node::new("127.0.0.1:12346".to_string());
    let mut app = Application::new();
    app.join(node);
    app.join(node2);
    
    Ok(())
}
