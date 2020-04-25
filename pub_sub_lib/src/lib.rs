#[macro_use]
extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;


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

#[derive(Clone)]
pub struct DBRepository {
    filepath: String
}

impl DBRepository {
    pub fn new(filepath: String) -> DBRepository {
        DBRepository{filepath: filepath}
    }

    pub fn save(self, key: String,  info: HashMap<String, String>){
        let parsed_info = serde_json::to_string(&info).unwrap();
        let db = DB::open_default(self.filepath).unwrap();
        db.put(key, parsed_info).unwrap(); 
    }

    pub fn get(self, key: String) -> Option<HashMap<String, String>> {
        let db = DB::open_default(self.filepath).unwrap();
        let ret =  match db.get(key.clone()) {
            Ok(Some(value)) =>  {
                let tmp_val = String::from_utf8(value).unwrap();
                let str_result = tmp_val.as_str();
                Some(serde_json::from_str(str_result).unwrap())
                },
            Ok(None) =>  None,
            Err(e) =>{ println!("operational problem encountered: {}", e); None},
        };
        let _ =  db.delete(key); 
        ret
    }
    pub fn contains(self, key: String) -> bool {
        let db = DB::open_default(self.filepath).unwrap();
        let ret = match db.get(key.clone()) {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => false,
        };
        ret  
    }

    pub fn remove(self, key: String) -> bool{
        let db = DB::open_default(self.filepath).unwrap();
        let ret = match db.delete(key.clone()) {
            Ok(_) => true,
            Err(_) => false,
        };
        ret
    }
}



pub trait JSONMessage {
    fn to_json(&self) -> Result<String, serde_json::Error>;
    fn get_operation(self) -> String;
}


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

    pub fn unsuscribe(topic: String, user: String) -> Message {
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
#[derive(Clone)]
pub struct Server;





impl Server {
    pub async fn run(self, address: String, user: String, replier: Arc<Mutex<Box<dyn MessageReplier>>>) -> Result<(), Box<dyn Error>> { 
        println!("Trying to connect to {}", address)    ;

        let addr = address.as_str().parse::<SocketAddr>()?;

        let mut listener = TcpListener::bind(&addr).await?; 
        loop  {

                let replier = replier.clone();
                let user = user.clone();
                let address = address.clone();

                println!("Wait for a new socket...");
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
                                    let _repl = replier.lock().unwrap(); 
                                    let _manager = MessageManager::new((*_repl).box_clone());
                                    let str_message = String::from_utf8(message).unwrap();
                                    match  _manager.exec(str_message) {
                                        Some(response) => { response_message = response}
                                        None => {println!("It is not necessary to reply the message")}
                                    };
                                };
                                    framed_writer.send(response_message.to_json().unwrap().as_bytes().to_vec())
                                                   .await.map_err(|e| println!("not response! {}", e)).ok();
                          }
                            Err(e) => {
                                println!("Error received while we are reading {}", e);
                            }

                        }
                    }
            });
        }
    }
}

pub async fn send(address: String, mesg: String) -> Result<Box<Message>, Box<dyn Error>> {
    println!("Trying to connect to {}", address);
    let remote_address: SocketAddr = address.parse().unwrap();
    let mut tcp = TcpStream::connect(&remote_address).await?;
    let (r, w) = tcp.split();
    
    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
    let mut framed_reader = FramedRead::new(r, MyBytesCodec{});
    
    let encoded: Vec<u8> = mesg.as_bytes().to_vec();
    framed_writer.send(encoded).await?;
    println!("It is a correct response");

    if let Some(frame) = framed_reader.next().await {
        match frame {
            Ok(response) => {
                println!("I got a response");
                println!("{:?}", response);
                let str_messg= String::from_utf8(response).unwrap();
                let messg: Message  = serde_json::from_str(&str_messg).unwrap();
                return Ok(Box::new(messg))
            }
            Err(e) => {
                println!("Error received while we are reading {}", e);
                return Err(Box::new(e))
            }

        }
    } else  {
        return Err(Box::new(std::io::Error::new(ErrorKind::Other, "uchs")))
    }
}


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

pub struct MessageManager {
    replier: Box<dyn MessageReplier>,
}

impl MessageManager  {
    fn new(replier: Box<dyn MessageReplier>) -> MessageManager {
        MessageManager{replier: replier}
    }
    fn exec(self, str_messg: String) -> Option<Box<Message>> {
        let messg: Message  = serde_json::from_str(&str_messg).unwrap();
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




#[cfg(test)]
mod tests {
        use super::*;
    #[test]
    fn it_should_parse_an_ack_correctly() {
        let mut addresses: HashMap<String, String> = HashMap::new();
        addresses.insert("user1".to_string(),"127.0.0.1".to_string());
        let messg = Message::ack();
        let str_messg: String  = serde_json::to_string(&messg).unwrap();
        println!("{}", str_messg.as_str());
        assert_eq!("{\"operation\":\"ack\",\"channel\":\"\",\"info\":{},\"mesg\":\"\"}", str_messg.as_str())
    }

    #[test]
    fn json_manager_should_parse_correctly_login_message() {
        let _message_manager = MessageManager::new();
        let vec_messg = Message::ack().to_json().unwrap().as_bytes().to_vec();

        let message = String::from_utf8(vec_messg).unwrap();
        println!("Json parser for: {:?}", message);
        _message_manager.exec(message);
    }


}

