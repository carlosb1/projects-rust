#[macro_use]
extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;


use bytes::BytesMut;
use std::io;
use tokio_util::codec::{Encoder,Decoder};
use tokio_util::codec::{FramedWrite, FramedRead};
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::error::Error;
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;



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


#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    operation: String,
    info: HashMap<String, String>
}

impl Message {
    pub fn new(operation: String) -> Message {
        Message{operation: operation, info: HashMap::new()}
    }

    pub fn ack() -> Message {
        Message{operation: "ack".to_string(), info: HashMap::new()}
    }

    pub fn nack(error_info: String) -> Message {
        let mut info: HashMap<String, String> =  HashMap::new();
        info.insert("error".to_string(), error_info.to_string());
        Message{operation: "nack".to_string(), info }
    }

    pub fn login(channel: String, addresses: HashMap<String, String>) -> Message {
        Message{operation: "login".to_string(), info: addresses}
    }
    pub fn ack_login(addresses: HashMap<String, String>) -> Message {
        Message {operation: "ack_login".to_string(), info: addresses}
    }

    pub fn send_msg(msg: String, channel: String, target_user: String) -> Message { 
        let mut info: HashMap<String, String> =  HashMap::new();
        info.insert("channel".to_string(), channel);
        info.insert("target_user".to_string(), target_user); 
        Message {operation: "send".to_string(), info: info} 
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

pub async fn send(address: String, message: Message) -> Result<(), Box<dyn Error>> {
    println!("Trying to connect to {}", address);
    let remote_address: SocketAddr = address.parse().unwrap();
    let mut tcp = TcpStream::connect(&remote_address).await?;
    let (r, w) = tcp.split();
    
    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
    let mut framed_reader = FramedRead::new(r, MyBytesCodec{});
    
    let encoded: Vec<u8> = serde_json::to_vec(&message).unwrap();
    framed_writer.send(encoded).await?;

    if let Some(frame) = framed_reader.next().await {
        match frame {
            Ok(response) => {
                let json_parser = JSONParser::new();
                //println!("{:?}", response);
                json_parser.parse(&response);
                let resp: Vec<u8>  = vec![1,2];
                framed_writer.send(resp).await.map_err(|e| println!("not response! {}", e)).ok();
            }
            Err(e) => {
                println!("Error received while we are reading {}", e);
            }

        }
    }
    Ok(()) 
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
        assert_eq!("{\"operation\":\"ack\",\"info\":{}}", str_messg.as_str())
    }

}

