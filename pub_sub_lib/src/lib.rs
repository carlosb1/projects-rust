#[macro_use]
extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;
extern crate pretty_env_logger;
extern crate log;


use tokio::runtime::Runtime;
use bytes::BytesMut;
use std::io;
use tokio::net::TcpListener;
use tokio_util::codec::{Encoder,Decoder};
use tokio_util::codec::{FramedWrite, FramedRead};
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::error::Error;
use std::io::{ErrorKind};
use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use rocksdb::DB;
use log::{info, error};

/// User interface trait. It specifies a contract of functions to be used as User interface.
pub trait UserInterface: Send + Sync{
    fn show(self, topic: String, msg: String);
}

/// CLI implementation for a CLI User interface
#[derive(Clone)]
pub struct CLI;

impl UserInterface for CLI  {
    fn show(self, topic: String, msg: String) {
        info!("{} {}", topic, msg);
    }
}

/// DB repository pattern to save our shared state... subscriptions, addresses, etc...
#[derive(Clone)]
pub struct DBRepository {
    filepath: String
}

impl DBRepository{
    pub fn new(filepath: String) -> DBRepository {
        DBRepository{filepath: filepath}
    }

    pub fn save(self, key: &str,  info: HashMap<String, String>){
        let parsed_info = serde_json::to_string(&info).expect("It was not possible parse info correctly from json.");
        let db = DB::open_default(self.filepath).expect("It was not possible to open db file.");
        db.put(key, parsed_info).expect("It was not possible put info in the db"); 
    }

    pub fn get(self, key: &str) -> Option<HashMap<String, String>> {
        let db = DB::open_default(self.filepath).expect("It was not possible to open db file.");
        let ret =  match db.get(key) {
            Ok(Some(value)) =>  {
                let tmp_val = String::from_utf8(value).expect("It was not possible parse db value.");
                let str_result = tmp_val.as_str();
                Some(serde_json::from_str(str_result).expect("It was not possible to parse from json in the db."))
                },
            Ok(None) =>  None,
            Err(e) =>{ error!("operational problem encountered: {}", e); None},
        };
        let _ =  db.delete(key); 
        ret
    }
    pub fn contains(self, key: &str) -> bool {
        let db = DB::open_default(self.filepath).expect("It was not possible to open db file.");
        let ret = match db.get(key) {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => false,
        };
        ret  
    }

    pub fn remove(self, key: &str) -> bool{
        let db = DB::open_default(self.filepath).expect("It was not possible to open db file.");
        let ret = match db.delete(key) {
            Ok(_) => true,
            Err(_) => false,
        };
        ret
    }
}


/// Trait for JSON message. Function contracts for serialize messages.
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
    pub mesg: String
}

impl JSONMessage for Message {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
    fn get_operation(self) -> String {
        self.operation
    }
}

/// Implement factory functions for each type message.
impl Message {
    pub fn new(operation: String) -> Message {
        Message{operation: operation, ..Default::default()}
    }

    pub fn ack(user: String,  address_source: String) -> Message {
        let mut info: HashMap<String, String> =  HashMap::new();
        info.insert(user,address_source);
        Message{operation: "ack".to_string(), ..Default::default()}
    }

    pub fn nack(error_info: String) -> Message {
        let mut info: HashMap<String, String> =  HashMap::new();
        info.insert("error".to_string(), error_info.to_string());
        Message{operation: "nack".to_string(), info, ..Default::default()}
    }

    pub fn subscribe(topic: String, user: String,  address_source: String) -> Message {
        let mut addresses: HashMap<String, String> =  HashMap::new();
        addresses.insert(user, address_source);
        Message{operation: "subscribe".to_string(), topic: topic,  info: addresses, ..Default::default()}
    }

    pub fn unsubscribe(topic: String, user: String) -> Message {
        let mut addresses: HashMap<String, String> =  HashMap::new();
        addresses.remove(&user);
        Message{operation: "unsubscribe".to_string(), topic: topic,  info: addresses, ..Default::default()}
    }

    pub fn ack_subscribe(topic: String, addresses: HashMap<String, String>) -> Message {
        Message {operation: "ack-subscribe".to_string(), topic: topic, info: addresses, ..Default::default()}
    }

    pub fn notify(msg: String, topic: String) -> Message { 
        Message {operation: "send".to_string(), topic: topic, mesg: msg, ..Default::default()} 
    }

}

/// Byte encoder / decoder for Tokio.
pub struct MyBytesCodec;

    
impl Decoder for MyBytesCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Vec<u8>>> {
        if buf.len() == 0 {
            return Ok(None);
        }
        let data = buf.clone().to_vec();
        buf.clear();
        Ok(Some(data))
    }
}


impl Encoder for MyBytesCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn encode(&mut self, data: Vec<u8>, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(data);
        Ok(())
    }
}


/// Server TCP implementation for tokio.
#[derive(Clone)]
pub struct Server;



impl Server {
    pub async fn run(self, address: String, user: String, replier: Arc<Mutex<Box<dyn MessageReplier>>>) -> Result<(), Box<dyn Error>> { 
        info!("Trying to connect to {}", address);

        let addr = address.as_str().parse::<SocketAddr>()?;

        let mut listener = TcpListener::bind(&addr).await?; 
        loop  {

                let replier = replier.clone();
                let user = user.clone();
                let address = address.clone();

                info!("Wait for a new socket...");
                let (mut socket, _) = listener.accept().await?;
                tokio::spawn(async move {                 
                    let (r, w)  = socket.split();
                    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
                    let mut framed_reader = FramedRead::new(r, MyBytesCodec{});
    
                   if let Some(frame) = framed_reader.next().await {
                        match frame {
                             Ok(message) => {
                                let mut response_message = Box::new(Message::ack(user, address));
                                let _ = {
                                    let _repl = replier.lock().expect("It was not possible to unlock shared replier message"); 
                                    let _manager = MessageManager::new((*_repl).box_clone());
                                    let str_message = String::from_utf8(message).expect("It was not possible to parse message to a string");
                                    match  _manager.exec(str_message) {
                                        Some(response) => { response_message = response}
                                        None => {info!("It is not necessary to reply the message")}
                                    };
                                };
                                    framed_writer.send(response_message.to_json().expect("Error parsing json message").as_bytes().to_vec())
                                                       .await.map_err(|e| println!("not response! {}", e)).ok();
                          }
                            Err(e) => {
                                error!("Error received while we are reading {}", e);
                            }

                        }
                    }
            });
        }
    }
}

/// Send function for tokio. It sends json messages.
pub async fn send(address: String, mesg: String) -> Result<Box<Message>, Box<dyn Error>> {
    info!("Trying to connect to {}", address);
    let remote_address: SocketAddr = address.parse().expect("it was not possible to parse net address");
    let mut tcp = TcpStream::connect(&remote_address).await?;
    let (r, w) = tcp.split();
    
    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
    let mut framed_reader = FramedRead::new(r, MyBytesCodec{});
    
    let encoded: Vec<u8> = mesg.as_bytes().to_vec();
    framed_writer.send(encoded).await?;
    info!("It received a response");

    if let Some(frame) = framed_reader.next().await {
        match frame {
            Ok(response) => {
                let str_messg= String::from_utf8(response).expect("It was not possible to parse message to a string");
                info!("{:?}", str_messg);
                let messg: Message  = serde_json::from_str(&str_messg).expect("It was not parsed json message to Message");
                return Ok(Box::new(messg))
            }
            Err(e) => {
                error!("Error received while we are reading {}", e);
                return Err(Box::new(e))
            }

        }
    } else  {
        return Err(Box::new(std::io::Error::new(ErrorKind::Other, "It was no possible to receive response from server")))
    }
}

/// Trait for replies. it includes trigger functions for each type of message.
pub trait MessageReplier: Send + Sync {
    fn on_ack(self: Box<Self>, messg: &Message);
    fn on_subscribe(self: Box<Self>, messg: &Message) -> Box<Message>;
    fn on_unsubscribe(self: Box<Self>, messg: &Message) -> Box<Message>;
    fn on_nack(self: Box<Self>, messg: &Message);
    fn on_ack_subscribe(self: Box<Self>, messg: &Message) -> Box<Message>;
    fn on_notify(self: Box<Self>, messg: &Message) -> Box<Message>;
    fn new_ack(self: Box<Self>) -> Box<Message>;
    fn box_clone(&self) -> Box<dyn MessageReplier>;
}

/// Dispatcher class for each type of responses.
pub struct MessageManager {
    replier: Box<dyn MessageReplier>,
}

impl MessageManager  {
    fn new(replier: Box<dyn MessageReplier>) -> MessageManager {
        MessageManager{replier: replier}
    }
    fn exec(self, str_messg: String) -> Option<Box<Message>> {
        let messg: Message  = serde_json::from_str(&str_messg).expect("It was not parsed json message to string");
        let oper = messg.operation.as_str();
        match oper {
            "ack" =>  {self.replier.on_ack(&messg); None},
            "nack" => {self.replier.on_nack(&messg); None},
            "ack-subscribe" => Some(self.replier.on_ack_subscribe(&messg)),
            "subscribe" => Some(self.replier.on_subscribe(&messg)),
            "unsubscribe" => Some(self.replier.on_unsubscribe(&messg)),
            "notify" => Some(self.replier.on_notify(&messg)),
            _ => {self.replier.new_ack(); None},
        }

    }
}

/// Main manager class. It uses client and server classes for a subscribe and publisher pattern.
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
    /// Init tcp server.
    pub fn init(&self) {
        let replier: Arc<Mutex<Box<dyn MessageReplier>>> = Arc::new(Mutex::new(Box::new((*self).clone())));
        let mut rt = Runtime::new().unwrap();
        let _ =  rt.block_on((*self).clone().server.run(self.address.clone(), self.user.clone(), replier));
        info!("Manager initialized");
    }

   /// Send subscription to a channel via a known client who is subscribed.
   pub fn subscribe(self, topic: String, seed_address: String) { 
        let mut rt = Runtime::new().unwrap();
        let message  = Message::subscribe(topic.to_string(),self.user.to_string(), self.address.to_string());
        info!("Send subscription message {}?", message.to_json().expect("Error parsing json message"));
        let result:  Result<Box<Message>, Box<dyn Error>>  = rt.block_on(send(seed_address, message.to_json().expect("Error parsing json message"))); 
        match result {
            Ok(message) =>{
                info!("Saving subscribe operation {}",message.to_json().expect("Error parsing json message").as_str());
                let users =  message.info.clone();
                self.db_info.save(topic.as_str(), users);
            },
           Err(e) => {
             error!("Error response from susbscribe {}?", e)
            },
        }
   }

   /// Notify operation for a topic where we are susbscribed.
   pub fn notify<'a>(self, topic: String, msg: String) -> Result<(), &'a str>  {
        let res = match self.db_info.get(topic.as_str()) {
            Some(entry) => {
                for (_, address) in entry.iter() {
                        let message = Message::notify(msg.clone(), topic.clone());
                        info!("Send notification message {}?", message.to_json().expect("Error parsing json message"));
                        let _ = send(address.clone(), message.to_json().expect("Error parsing json message").to_string());
                }
                Ok(())
            },
            None => { Err("It was not found")}
        };
        res

   }

   /// Unsubscribe from topic.
   pub fn unsubscribe<'a>(self, topic: String) -> Result<(), &'a str> {
        let res = match self.db_info.get(topic.as_str()) {
            Some(entry) => {
                for (user, address) in entry.iter() {
                        let message = Message::unsubscribe(topic.clone(), user.clone());
                        info!("Send unsubscribe message {}?", message.to_json().expect("Error parsing json message"));
                        let _ = send(address.clone(), message.to_json().expect("Error parsing json message").to_string());
                }
                Ok(())
            },
            None => { Err("It was not found")}
        };
        res
   }
}

/// Replier implementation for the manager. 
impl MessageReplier for Manager {
    fn on_ack(self: Box<Self>, _: &Message) {
        info!("Ack received");
    }
    fn on_subscribe(self: Box<Self>, messg: &Message)  -> Box<Message>{
        info!("susbcribed received");

        let mut users: HashMap<String, String> = match self.db_info.clone().get(messg.topic.as_str()) {
            Some(val) =>{val.clone()}
            None => { HashMap::new()}
        };
        for (user, addr) in messg.info.iter() {
            users.insert(user.clone(), addr.clone());
        }
        self.db_info.save(messg.topic.as_str(), users.clone());
        Box::new(Message::ack_subscribe(messg.topic.clone(), users.clone()))

    }
    fn on_unsubscribe(self: Box<Self>, messg: &Message)  -> Box<Message>{
        info!("Unsubscribed received");
        if let Some(mut user_entry) = self.db_info.clone().get(messg.topic.as_str()) {
            for (key, _) in messg.info.iter() {
                user_entry.remove(&key.clone());
            }
            self.db_info.save(messg.topic.as_str(), user_entry)
        }
        Box::new(Message::ack(self.user.clone(), self.address.clone()))
    }
    fn on_nack(self: Box<Self>, messg: &Message){
        info!("On Nack received");
        info!("Error message {}?", messg.info.get("error").unwrap_or(&"No available error".to_string()));
    }
    fn on_ack_subscribe(self: Box<Self>, messg: &Message) -> Box<Message>{
        info!("Ack Login received");
        self.db_info.save(messg.topic.as_str(), messg.info.clone());
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





#[cfg(test)]
mod tests {
        use super::*;
    #[test]
    fn it_should_parse_an_ack_correctly() {
        let mut addresses: HashMap<String, String> = HashMap::new();
        addresses.insert("user1".to_string(),"127.0.0.1".to_string());
        let messg = Message::ack();
        let str_messg: String  = serde_json::to_string(&messg).expect("It was not parsed json message to string");
        println!("{}", str_messg.as_str());
        assert_eq!("{\"operation\":\"ack\",\"channel\":\"\",\"info\":{},\"mesg\":\"\"}", str_messg.as_str())
    }

    #[test]
    fn json_manager_should_parse_correctly_login_message() {
        let _message_manager = MessageManager::new();
        let vec_messg = Message::ack().to_json().expect("Error parsing json message").as_bytes().to_vec();

        let message = String::from_utf8(vec_messg).unwrap();
        println!("Json parser for: {:?}", message);
        _message_manager.exec(message);
    }


}

