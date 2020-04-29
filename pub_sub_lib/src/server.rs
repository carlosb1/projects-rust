extern crate serde_derive;

use tokio::runtime::Runtime;
use pub_sub::{Server, MessageReplier, Message, UserInterface, CLI};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use log::info;


#[derive(Clone)]
pub struct MockReplier{
    pub subscriptions: HashMap<String, HashMap<String, String>>,
    user: String,
    address: String,
    interface: Box<CLI>
}

impl MockReplier {
    pub fn new(user: String, address: String) -> MockReplier {
        MockReplier{subscriptions: HashMap::new(), user: user, address: address, interface: Box::new(CLI{})}
    }
    
}

impl MessageReplier for MockReplier {
    fn on_ack(self: Box<Self>, _: &Message) {
        info!("Ack received");
    }
    fn on_subscribe(mut self: Box<Self>, messg: &Message)  -> Box<Message>{
        info!("susbcribed received");

        let mut users: HashMap<String, String> = match self.subscriptions.get(&messg.topic) {
            Some(val) =>{val.clone()}
            None => { HashMap::new()}
        };
        for (key, val) in messg.info.iter() {
            users.insert(key.clone(), val.clone());
        }
        self.subscriptions.insert(messg.topic.clone(), users.clone());
        Box::new(Message::ack_subscribe(messg.topic.clone(), users.clone()))

    }
    fn on_unsubscribe(mut self: Box<Self>, messg: &Message)  -> Box<Message>{
        info!("Unsubscribed received");
        if let Some(user_entry) = self.subscriptions.get_mut(&messg.topic) {
            for (key, _) in messg.info.iter() {
                info!("Unsuscribing {}", key);
                user_entry.remove(&key.clone());
            } 
        }
        Box::new(Message::ack(self.user.clone(), self.address.clone()))
    }
    fn on_nack(self: Box<Self>, messg: &Message){
        info!("On Nack received");
        info!("Error message {}?", messg.info.get("error").unwrap_or(&"No available error".to_string()));
    }
    fn on_ack_subscribe(mut self: Box<Self>, messg: &Message) -> Box<Message>{
        info!("Ack Login received");
        self.subscriptions.insert(messg.topic.clone(), messg.info.clone());
        Box::new(Message::ack(self.user, self.address))
    }
    fn on_notify(self: Box<Self>, messg: &Message) -> Box<Message>{
        info!("Notification received");
        let result_message = Message::ack(self.user, self.address);
        let mesg = messg.mesg.clone();
        let topic = messg.topic.clone();
        self.interface.show(topic, mesg);
        Box::new(result_message)
    }
    fn new_ack(self: Box<Self>) -> Box<Message> {
        Box::new(Message::ack(self.user, self.address))
    }
    fn box_clone(&self)-> Box<dyn MessageReplier> {
        Box::new((*self).clone()) 
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let mut rt = Runtime::new()?;
    
    
    let user = "user".to_string();
    let address = "127.0.0.1:12345".to_string();

    let mock_replier = MockReplier::new(user.clone(), address.clone());
    let replier: Arc<Mutex<Box<dyn MessageReplier>>> = Arc::new(Mutex::new(Box::new(mock_replier))); 
    let server = Server{};
    rt.block_on(server.run(address, user,replier))

}
