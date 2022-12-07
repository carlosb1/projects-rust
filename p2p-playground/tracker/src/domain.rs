use serde_derive::{Deserialize, Serialize};
use sha256::digest;
use std::collections::HashMap;
use std::sync::Arc;

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
            operation: "ack".to_string(),
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
    pub fn add(&mut self, node: &Node) {
        let hasher = Hasher {};
        let key = hasher.hash(format!("{:}-{:}", node.user, node.address));
        self.data.insert(key, (*node).clone());
    }
}

struct RegisterNewNode {
    repository: Arc<NodeRepository>,
}
impl RegisterNewNode {
    const USER_PARAM: &str = "user_param";
    const ADDRESS: &str = "address";

    pub fn new(repository: Arc<NodeRepository>) -> Self {
        RegisterNewNode { repository }
    }

    pub fn run(message: Message) -> Option<Message> {
        let user = message.info.get(&RegisterNewNode::USER_PARAM.to_string())?;
        let address = message.info.get(&RegisterNewNode::ADDRESS.to_string())?;
        let node = Node::new(user, address, Type::User);

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
        pub fn register_new_node_correctly() {}
    }
    mod devices {
        mod repositories {
            pub fn add_new_node() {}
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
