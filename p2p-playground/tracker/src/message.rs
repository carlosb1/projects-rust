use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

//// Trait for JSON message. Function contracts for serialize messages.
pub trait JSONMessage {
    fn to_json(&self) -> Result<String, serde_json::Error>;
    fn get_operation(self) -> String;
}

/// Message class for messages. It is serialize in a json message.
#[derive(Serialize, Deserialize, Debug, Default)]
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

    pub fn new_user(user: String, address_source: String) -> Message {
        let mut info: HashMap<String, String> = HashMap::new();
        info.insert(user, address_source);
        Message {
            operation: "ack".to_string(),
            ..Default::default()
        }
    }
}

impl JSONMessage for Message {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
    fn get_operation(self) -> String {
        self.operation
    }
}
