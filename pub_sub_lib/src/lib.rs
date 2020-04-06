#[macro_use]
extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;


use bytes::BytesMut;
use std::io;
use tokio_util::codec::{Encoder,Decoder};

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
