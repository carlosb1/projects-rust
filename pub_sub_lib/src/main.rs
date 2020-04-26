extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;
extern crate pub_sub;
extern crate pretty_env_logger;
extern crate log;



use tokio::runtime::Runtime;
use pub_sub::{Server, MessageReplier, Message, DBRepository, send, JSONMessage};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::error::Error;
use std::{thread, time};
use log::info;

pub trait UserInterface: Send + Sync{
    fn show(self, topic: String, msg: String);
}



#[derive(Clone)]
pub struct CLI;

impl UserInterface for CLI  {
    fn show(self, topic: String, msg: String) {
        info!("{} {}", topic, msg);
    }
}

#[derive(Clone)]
pub struct Manager{
    filepath_db: String,
    user: String, 
    address: String,
    server: Server,
    db_info: DBRepository,
    interface: Box<CLI>
}

impl Manager {
    pub fn new(filepath_db: String, user: String, address: String) -> Manager {
        Manager{filepath_db: filepath_db.clone(), user: user.clone(), address: address.clone(), server: Server{},db_info: DBRepository::new(filepath_db.clone()),interface: Box::new(CLI{})}
    }
    pub fn init(&self) {
        let replier: Arc<Mutex<Box<dyn MessageReplier>>> = Arc::new(Mutex::new(Box::new((*self).clone())));
        let mut rt = Runtime::new().unwrap();
        let _ =  rt.block_on((*self).clone().server.run(self.address.clone(), self.user.clone(), replier));
    }

   pub fn subscribe(self, topic: String, seed_address: String) { 
        let mut rt = Runtime::new().unwrap();
        let message  = Message::subscribe(topic.to_string(),self.user.to_string(), self.address.to_string());
        let result:  Result<Box<Message>, Box<dyn Error>>  = rt.block_on(send(seed_address, message.to_json().unwrap())); 
        match result {
            Ok(message) =>{
                let users =  message.info.clone();
                self.db_info.save(topic, users);
                println!("{}",message.to_json().unwrap().as_str());
            },
           Err(e) => {
             println!("{}?", e)
            },
        }
   }
   pub fn notify<'a>(self, topic: String, msg: String) -> Result<(), &'a str>  {
        let res = match self.db_info.get(topic.clone()) {
            Some(entry) => {
                for (_, address) in entry.iter() {
                        let message = Message::notify(msg.clone(), topic.clone());
                        let _ = send(address.clone(), message.to_json().unwrap().to_string());
                }
                Ok(())
            },
            None => { Err("It was not found")}
        };
        res

   }
   pub fn unsubscribe<'a>(self, topic: String) -> Result<(), &'a str> {
        let res = match self.db_info.get(topic.clone()) {
            Some(entry) => {
                for (user, address) in entry.iter() {
                        let message = Message::unsuscribe(topic.clone(), user.clone());
                        let _ = send(address.clone(), message.to_json().unwrap().to_string());
                }
                Ok(())
            },
            None => { Err("It was not found")}
        };
        res
   }
}


impl MessageReplier for Manager {
    fn on_ack(self: Box<Self>, _: &Message) {
        info!("Ack received");
    }
    fn on_subscribe(self: Box<Self>, messg: &Message)  -> Box<Message>{
        info!("susbcribed received");

        let mut users: HashMap<String, String> = match self.db_info.clone().get(messg.topic.clone()) {
            Some(val) =>{val.clone()}
            None => { HashMap::new()}
        };
        for (user, addr) in messg.info.iter() {
            users.insert(user.clone(), addr.clone());
        }
        self.db_info.save(messg.topic.clone(), users.clone());
        Box::new(Message::ack_subscribe(messg.topic.clone(), users.clone()))

    }
    fn on_unsubscribe(self: Box<Self>, messg: &Message)  -> Box<Message>{
        info!("Unsubscribed received");
        if let Some(mut user_entry) = self.db_info.clone().get(messg.topic.clone()) {
            for (key, _) in messg.info.iter() {
                user_entry.remove(&key.clone());
            }
            self.db_info.save(messg.topic.clone(), user_entry)
        }
        Box::new(Message::ack(self.user.clone(), self.address.clone()))
    }
    fn on_nack(self: Box<Self>, messg: &Message){
        info!("On Nack received");
        info!("Error message {}?", messg.info.get("error").unwrap_or(&"No available error".to_string()));
    }
    fn on_ack_subscribe(self: Box<Self>, messg: &Message) -> Box<Message>{
        info!("Ack Login received");
        self.db_info.save(messg.topic.clone(), messg.info.clone());
        Box::new(Message::ack(self.user, self.address))
    }
    fn on_notify(self: Box<Self>, messg: &Message) -> Box<Message>{
        info!("notification received");
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
    let user = "user".to_string();
    let address = "127.0.0.1:12345".to_string();
    let filepath_db = "infodb".to_string();
    let manager = Manager::new(filepath_db, user, address);
    manager.init();
    info!("It was initialized");
    let sec_times = time::Duration::from_secs(60);
    thread::sleep(sec_times);
    Ok(())
}
