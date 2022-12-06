use crate::domain::Message;
use dyn_clone::DynClone;
use std::collections::HashMap;

// - blockchain validate library
// - IPFS for searching storage
// - generate CIDs
// - generate tracker
//      - return a list of nodes, its IPS, mdns?

/// Trait for replies. it includes trigger functions for each type of message.
pub trait MessageReplier: DynClone {
    fn run(self: Box<Self>, messg: Message) -> Option<Message>;
}

dyn_clone::clone_trait_object!(MessageReplier);

/// Dispatcher class for each type of responses.
#[derive(Clone)]
pub struct MessageManager {
    pub replier: HashMap<String, Box<dyn MessageReplier>>,
}

impl MessageManager {
    pub fn new() -> MessageManager {
        MessageManager {
            replier: HashMap::new(),
        }
    }
    pub fn exec(&mut self, str_messg: String) -> Option<Message> {
        let messg: Message =
            serde_json::from_str(&str_messg).expect("It was not parsed json message to string");
        let oper = messg.operation.as_str();
        self.replier
            .get(oper)
            .map_or(None, |repl| repl.clone().run(messg))
    }
}
unsafe impl Send for MessageManager {}
unsafe impl Sync for MessageManager {}
