use serde_derive::{Deserialize, Serialize};
use sha256::digest;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct P2pData {
    username: String,
    address: String,
}

struct P2pNetwork {}

impl P2pNetwork {
    pub fn join() {}
    pub fn leave() {}
    pub fn put() {}
    pub fn get() {}
}

/// Message class for messages. It is serialize in a json message.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Message {
    pub operation: String,
    pub topic: String,
    pub info: HashMap<String, String>,
    pub mesg: String,
}

impl Message {
    pub fn new(operation: String) -> Message {
        Message {
            operation,
            ..Default::default()
        }
    }
    pub fn ok() -> Message {
        Message {
            operation: "ok".to_string(),
            ..Default::default()
        }
    }

    pub fn new_user(user: String, address_source: String) -> Message {
        let mut info: HashMap<String, String> = HashMap::new();
        info.insert(user, address_source);
        Message {
            operation: "registry".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub enum Type {
    User,
    File,
}

#[derive(Clone)]
struct Node {
    typ: Type,
    user: String,
    address: String,
}
impl Node {
    pub fn new(user: &str, address: &str, typ: Type) -> Self {
        Node {
            user: user.to_string(),
            address: address.to_string(),
            typ,
        }
    }
}

use std::collections::BTreeMap;

#[derive(Clone)]
struct Hasher {}
impl Hasher {
    pub fn hash(self, info: String) -> String {
        digest(info).to_string()
    }
}

//TODO move to devices
struct NodeRepository {
    data: BTreeMap<String, Node>,
}

impl NodeRepository {
    pub fn new() -> Self {
        NodeRepository {
            data: BTreeMap::new(),
        }
    }
    pub fn add(&mut self, key: String, node: &Node) {
        let _ = self.data.insert(key, (*node).clone());
    }
}

struct RegisterNewUser {
    repository: Arc<Mutex<NodeRepository>>,
    hasher: Hasher,
}
impl RegisterNewUser {
    const USER_PARAM: &str = "user_param";
    const ADDRESS: &str = "address";

    pub fn new(repository: Arc<Mutex<NodeRepository>>) -> Self {
        RegisterNewUser {
            repository,
            hasher: Hasher {},
        }
    }

    pub fn run(&mut self, message: Message) -> Option<Message> {
        let user = message.info.get(&RegisterNewUser::USER_PARAM.to_string())?;
        let address = message.info.get(&RegisterNewUser::ADDRESS.to_string())?;
        let node = Node::new(user, address, Type::User);

        let key = self
            .hasher
            .clone()
            .hash(format!("{:}-{:}", node.user, node.address));

        self.repository.lock().unwrap().add(key, &node);
        Some(Message::ok())
    }
}

struct SearchNewUser {
    repository: Arc<Mutex<NodeRepository>>,
}

impl SearchNewUser {
    const USER_PARAM: &str = "user_param";

    pub fn new(repository: Arc<Mutex<NodeRepository>>) -> Self {
        SearchNewUser { repository }
    }
    pub fn run(&mut self, message: Message) -> Option<Message> {
        let user = message.info.get(&SearchNewUser::USER_PARAM.to_string())?;
        Some(Message::ok())
    }
}

#[cfg(test)]
mod tests {
    mod messages {
        use crate::domain::*;
        use pretty_assertions::assert_eq;
        use rstest::*;
        #[rstest]
        pub fn ok_should_be_correctly() {
            assert_eq!(Message::ok().operation, "ok".to_string());
        }
    }
    mod cases {
        use crate::domain::Message;
        use crate::domain::NodeRepository;
        use crate::domain::RegisterNewUser;
        use rstest::*;
        use std::sync::{Arc, Mutex};

        #[rstest]
        pub fn register_new_user_node_correctly() {
            let repository = Arc::new(Mutex::new(NodeRepository::new()));
            let mut new_node_case = RegisterNewUser::new(repository);
            let message = Message::new_user("user".to_string(), "myaddress".to_string());
            new_node_case.run(message);
        }
    }
    mod devices {
        mod repositories {
            use crate::domain::{Hasher, Node, NodeRepository, Type};
            use rstest::*;

            #[rstest]
            pub fn add_new_node() {
                let mut repository = NodeRepository::new();
                let node = Node::new("user", "address", Type::User);
                let key = Hasher {}.hash(format!("{:}-{:}", node.user, node.address));
                repository.add(key, &node);
                assert_eq!(1, repository.data.len());
            }
            #[rstest]
            pub fn not_add_existing_node() {
                let mut repository = NodeRepository::new();
                let node = Node::new("user", "address", Type::User);
                let key = Hasher {}.hash(format!("{:}-{:}", node.user, node.address));
                repository.add(key.clone(), &node);

                let node = Node::new("user", "address", Type::User);
                repository.add(key.clone(), &node);
                assert_eq!(1, repository.data.len());
            }
        }
        mod hashers {
            use crate::domain::Hasher;
            use rstest::*;
            #[rstest]
            pub fn new_hash() {
                let hasher = Hasher {};
                assert_eq!(
                    "50d858e0985ecc7f60418aaf0cc5ab587f42c2570a884095a9e8ccacd0f6545c",
                    hasher.hash("example".to_string())
                );
            }
        }
    }
}
