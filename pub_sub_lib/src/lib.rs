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
}

impl Message {
    pub fn new(operation: String) -> Message {
        Message{operation: operation}
    }

    pub fn ack() -> Message {
        Message{operation: "ack".to_string()}
    }

    pub fn nack() -> Message {
        Message{operation: "nack".to_string()}
    }

    pub fn login(channel: String, users: HashMap<&str,Vec<&str>>) -> Message {
        Message{operation: "login".to_string()}
    }
    pub fn ack_login(users: HashMap<&str,Vec<&str>>) -> Message {
        Message {operation: "ack_login".to_string()}
    }

    pub fn send_msg(msg: String) -> Message {
        Message {operation: "send".to_string()} 
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
