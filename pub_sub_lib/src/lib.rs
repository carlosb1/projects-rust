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
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


pub trait JSONMessage {
    fn to_json(&self) -> Result<String, serde_json::Error>;
    fn get_operation(self) -> String;
}


#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Message {
    operation: String,
    channel: String,
    info: HashMap<String, String>,
    mesg: String
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

    pub fn ack() -> Message {
        Message{operation: "ack".to_string(), ..Default::default()}
    }

    pub fn nack(error_info: String) -> Message {
        let mut info: HashMap<String, String> =  HashMap::new();
        info.insert("error".to_string(), error_info.to_string());
        Message{operation: "nack".to_string(), info, ..Default::default()}
    }

    pub fn login(channel: String, addresses: HashMap<String, String>) -> Message {
        Message{operation: "login".to_string(), channel: channel,  info: addresses, ..Default::default()}
    }
    pub fn ack_login(addresses: HashMap<String, String>) -> Message {
        Message {operation: "ack-login".to_string(), info: addresses, ..Default::default()}
    }

    pub fn send_msg(msg: String, channel: String) -> Message { 
        Message {operation: "send".to_string(), channel: channel, mesg: msg, ..Default::default()} 
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
pub struct Server;

#[derive(Copy, Clone)]
struct Foo {
   num: i32 
}
impl Foo {
    pub fn hello(self) {
        println!("hello world");
    }
}



impl Server {
    pub async fn run(self, address: String) -> Result<(), Box<dyn Error>> { 
        println!("Trying to connect to {}", address)    ;
        let addr = address.as_str().parse::<SocketAddr>()?;
            

        let foo = Foo{num:0};

        let mutex = std::sync::Mutex::new(foo);
        let arc = std::sync::Arc::new(mutex);
    
        let replier = Arc::new(Mutex::new(MockReplier::new()));
        
        let gener_replier: Arc<Mutex<Box<dyn MessageReplier>>> = Arc::new(Mutex::new(Box::new(MockReplier2::new())));

        let mut listener = TcpListener::bind(&addr).await?; 
        loop  {



                let arc = arc.clone();
                let replier = replier.clone();
                let gener_replier = gener_replier.clone();



                println!("Wait for a new socket...");
                let (mut socket, _) = listener.accept().await?;
                tokio::spawn(async move {                 
                    let y = {
                        let x = arc.lock().unwrap();
                        (*x).hello();
                    };

 
                    //_ref_replier.lock().unwrap().on_ack(&Message::ack()); 
                    //(*_ref_replier).lock().unwrap().on_ack(&Message::ack()); 
                    // replier.on_login(&Message::ack());
                        let (r, w)  = socket.split();
                    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
                    let mut framed_reader = FramedRead::new(r, MyBytesCodec{});
    
                   if let Some(frame) = framed_reader.next().await {
                        match frame {
                            Ok(message) => {
                                //let json_parser = JSONParser::new();
                                //println!("{:?}", message);
                                //json_parser.parse(&message);
                                
                                 let z = {
                                    let unwrap_replier = replier.lock().unwrap();
                                     //(*unwrap_replier).on_ack(&Message::ack());
                                    //let _manager = MessageManager::new(*unwrap_replier);
                                    //let str_message = String::from_utf8(message).unwrap();
                                    //_manager.exec(str_message);
                                 };
                                let y2 = {
                                    let x2 = gener_replier.lock().unwrap(); 
                                    //(*x2).box_clone().on_ack(&Message::ack());
                                    
                                    let _manager = MessageManager2::new((*x2).box_clone());
                                    let str_message = String::from_utf8(message).unwrap();
                                    _manager.exec(str_message);
                                
                                };

//                                let str_message = String::from_utf8(message).unwrap();
                                //_manager.exec(str_message);
                                let response_message = Message::ack();
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

pub async fn send(address: String, mesg: String) -> Result<(), Box<dyn Error>> {
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
            }
            Err(e) => {
                println!("Error received while we are reading {}", e);
            }

        }
    }
    Ok(()) 
}


pub trait MessageReplier: Send + Sync {
    fn on_ack(self: Box<Self>, messg: &Message);
    fn hello(self: Box<Self>);
    fn box_clone(&self) -> Box<MessageReplier>;
//    fn on_login(self, messg: &Message);
//    fn on_nack(self, messg: &Message);
//    fn on_ack_login(self, messg: &Message);
//    fn on_send(self, messg: &Message);
}

pub struct MessageManager {
    replier: MockReplier
}

impl MessageManager  {
    fn new(replier: MockReplier) -> MessageManager {
        MessageManager{replier: replier}
    }
    fn exec(self, str_messg: String) {
        let messg: Message  = serde_json::from_str(&str_messg).unwrap();
        let oper = messg.operation.as_str();
        match oper {
            //"ack" => self.replier.on_ack(&messg),
            "login" => self.response_ack(&messg),
            "nack" => self.response_ack(&messg),
            "ack-login" => self.response_ack(&messg),
            "send" => self.response_ack(&messg),
            _ => println!("incorrect operation"),
        }

    }

    fn response_ack(self, messg: &Message) {
            println!("Hello ack message");
    }
        
}

pub struct MessageManager2 {
    replier: Box<dyn MessageReplier>
}

impl MessageManager2  {
    fn new(replier: Box<dyn MessageReplier>) -> MessageManager2 {
        MessageManager2{replier: replier}
    }
    fn exec(self, str_messg: String) {
        let messg: Message  = serde_json::from_str(&str_messg).unwrap();
        let oper = messg.operation.as_str();
        match oper {
            "ack" => self.replier.on_ack(&messg),
            _ => println!("incorrect operation"),
        }

    }

    fn response_ack(self, messg: &Message) {
            println!("Hello ack message");
    }
        
}

#[derive(Copy, Clone)]
pub struct MockReplier;

impl MockReplier {
    fn new() -> MockReplier {
        MockReplier{}
    }
}

impl MockReplier {
    pub fn on_ack(self, messg: &Message) {
        println!("Ack received");
    }
    pub fn on_login(self, messg: &Message) {
        println!("Login received");
    }
    pub fn on_nack(self, messg: &Message) {
        println!("NAck received");
    }
    fn on_ack_login(self, messg: &Message) {
        println!("Ack login received");
    }
    fn on_send(self, messg: &Message){
        println!("Send received");
    }
}

#[derive(Copy, Clone)]
pub struct MockReplier2;

impl MockReplier2 {
    fn new() -> MockReplier2 {
        MockReplier2{}
    }
}

impl MessageReplier for MockReplier2 {
    fn on_ack(self: Box<Self>, messg: &Message) {
        println!("Ack received");
    }
    fn hello(self: Box<Self>)  {
        println!("hello world");
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

