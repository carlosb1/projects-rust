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

/*
#[derive(Clone,Copy)]
pub struct JSONParser;
impl JSONParser {
    pub fn new() -> JSONParser {
        JSONParser{}
    }
}

impl JSONParser {
    pub fn parse(&self, info: &Vec<u8>) -> Option<Message> {
        let vec_to_parse = info.clone();
        let message = String::from_utf8(vec_to_parse).unwrap();
        println!("Json parser for: {:?}", message);
        serde_json::from_str(&message).ok()
    }
}
*/

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

impl Server {
    pub async fn run(self, address: String) -> Result<(), Box<dyn Error>> { 
        println!("Trying to connect to {}", address)    ;
        let addr = address.as_str().parse::<SocketAddr>()?;
        
        // let _manager: Arc<MessageManager> = Arc::new(MessageManager::new());
        let _manager: MessageManager = MessageManager::new();

        let mut listener = TcpListener::bind(&addr).await?; 
        loop  {
                println!("Wait for a new socket...");
                let (mut socket, _) = listener.accept().await?;
                tokio::spawn(async move {
                    let (r, w)  = socket.split();
                    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
                    let mut framed_reader = FramedRead::new(r, MyBytesCodec{});
    
                   if let Some(frame) = framed_reader.next().await {
                        match frame {
                            Ok(message) => {
                                //let json_parser = JSONParser::new();
                                //println!("{:?}", message);
                                //json_parser.parse(&message);
                                let str_message = String::from_utf8(message).unwrap();
                                _manager.exec(str_message);
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

#[derive(Clone,Copy)]
pub struct MessageManager;

impl MessageManager  {
    fn new() -> MessageManager {
        MessageManager{}
    }
    fn exec(self, str_messg: String) {
        let messg: Message  = serde_json::from_str(&str_messg).unwrap();
        let oper = messg.operation.as_str();
        match oper {
            "ack" => println!("Hello ack message"),
            "login" => println!("Hello login message"),
            "nack" => println!("Hello nack message"),
            "ack-login" => println!("Hello ack login message"),
            "send" => println!("Hello send message"),
            _ => println!("incorrect operation"),
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

