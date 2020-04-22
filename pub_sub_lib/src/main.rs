extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;
extern crate pub_sub;


use tokio::runtime::Runtime;
use pub_sub::{Server, MessageReplier, Message, DBRepository};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;




pub trait UserInterface: Send + Sync{
    fn show(self, topic: String, msg: String);
}



#[derive(Clone)]
pub struct CLI;

impl UserInterface for CLI  {
    fn show(self, topic: String, msg: String) {
        println!("{} {}", topic, msg);
    }
}



#[derive(Clone)]
pub struct Replier{
    dbInfo: DBRepository,
    user: String,
    address: String,
    interface: Box<CLI>
}

impl Replier {
    pub fn new(user: String, address: String) -> Replier {
        Replier{dbInfo: DBRepository::new("statusdb".to_string()), user: user, address: address, interface: Box::new(CLI{})}
    }
    
}

impl MessageReplier for Replier {
    fn on_ack(self: Box<Self>, _: &Message) {
        println!("Ack received");
    }
    fn on_subscribe(mut self: Box<Self>, messg: &Message)  -> Box<Message>{
        println!("susbcribed received");

        let mut users: HashMap<String, String> = match self.dbInfo.clone().get(messg.topic.clone()) {
            Some(val) =>{val.clone()}
            None => { HashMap::new()}
        };
        for (key, val) in messg.info.iter() {
            users.insert(key.clone(), val.clone());
        }
        self.dbInfo.save(messg.topic.clone(), users.clone());
        Box::new(Message::ack_subscribe(messg.topic.clone(), users.clone()))

    }
    fn on_unsubscribe(mut self: Box<Self>, messg: &Message)  -> Box<Message>{
        println!("Unsubscribed received");
        if let Some(mut user_entry) = self.dbInfo.clone().get(messg.topic.clone()) {
            for (key, _) in messg.info.iter() {
                user_entry.remove(&key.clone());
            }
            self.dbInfo.save(messg.topic.clone(), user_entry)
        }
        Box::new(Message::ack(self.user.clone(), self.address.clone()))
    }
    fn on_nack(self: Box<Self>, messg: &Message){
        println!("On Nack received");
        println!("Error message {}?", messg.info.get("error").unwrap_or(&"No available error".to_string()));
    }
    fn on_ack_subscribe(mut self: Box<Self>, messg: &Message) -> Box<Message>{
        println!("Ack Login received");
        self.dbInfo.save(messg.topic.clone(), messg.info.clone());
        Box::new(Message::ack(self.user, self.address))
    }
    fn on_notify(self: Box<Self>, messg: &Message) -> Box<Message>{
        println!("notification received");
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
    let mut rt = Runtime::new()?;
    
    let user = "user".to_string();
    let address = "127.0.0.1:12345".to_string();

    let mock_replier = Replier::new(user.clone(), address.clone());
    let replier: Arc<Mutex<Box<dyn MessageReplier>>> = Arc::new(Mutex::new(Box::new(mock_replier)));
    
    //mock_replier.subscriptions.clone().get("user");

    let server = Server{};
    rt.block_on(server.run(user, address,replier))

}
