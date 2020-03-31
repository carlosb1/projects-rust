#[macro_use]
extern crate serde_derive;

extern crate bytes;
extern crate tokio;


extern crate serde;
extern crate serde_json;

use bytes::BytesMut;
use std::io;
use tokio_util::codec::{Encoder,Decoder, Framed, FramedWrite, FramedRead};
use tokio::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::net::SocketAddr;
use std::error::Error;
use futures::{SinkExt};



/* generic interface for protocols */
pub trait MessageProtocolParser {
    fn parse(&self, info: &Vec<u8>);
    fn is_message(&self, info: &Vec<u8>) -> bool;
}


#[derive(Clone,Copy)]
pub struct ExampleJSONParser;
impl ExampleJSONParser {
    fn new() -> ExampleJSONParser {
        ExampleJSONParser{}
    }
}

impl MessageProtocolParser for ExampleJSONParser {
    fn is_message(&self, info: &Vec<u8>) -> bool {
        true
    }
    fn parse(&self, info: &Vec<u8>) {
        let vec_to_parse = info.clone();
        let message = String::from_utf8(vec_to_parse).unwrap();
        println!("Json parser for: {:?}", message);
        let msg: Message = match serde_json::from_str(&message)  {
            Err(..) =>   {println!("It was not parsed correctly"); Message::new_empty() },
            Ok(msg) => msg,
        };
    }
}



#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    operation: String,
}

impl Message {
    fn new(operation: String) -> Message {
        Message {operation: operation} 
    }
    fn new_empty() -> Message  {
        Message {operation: "".to_string()}
    }
    
}

pub struct MyBytesCodec{
    pub parsers: Vec<Arc<Mutex<Box<MessageProtocolParser+Send>>>>,
    pub vector_test: Vec<u8>,
}

impl MyBytesCodec {
    fn new(parsers: Vec<Arc<Mutex<Box<MessageProtocolParser+Send>>>>) -> MyBytesCodec {
        MyBytesCodec{parsers: parsers,vector_test:  Vec::new()}
    }
}
    
impl Decoder for MyBytesCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<Vec<u8>>> {
        if buf.len() == 0 {
            return Ok(None);
        }
        let data = buf.clone().to_vec();
        for parser in self.parsers.iter() {
            let cloned_data = data.clone();
            if parser.lock().unwrap().is_message(&cloned_data) {
                parser.lock().unwrap().parse(&cloned_data);
            }
        }
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
   
type ClientTransport = Framed<TcpStream, MyBytesCodec>;

pub async fn run(address: String) -> Result<(), Box<dyn Error>> {

    let remote_address: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut tcp = TcpStream::connect(&remote_address).await?;
    let (r, w) = tcp.split();
    
    let json_parser = ExampleJSONParser::new();
    let parsers: Vec<Arc<Mutex<Box<MessageProtocolParser+Send>>>>  = vec![Arc::new(Mutex::new(Box::new(json_parser)))];
    let mut framed_writer = FramedWrite::new(w, MyBytesCodec::new(parsers));
    
    //let mut framed_reader = FramedRead::new(r, MyBytesCodec::new(parsers));
    let encoded: Vec<u8> = vec![1,1];
    framed_writer.send(encoded).await?;
    Ok(()) 
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:12345".parse::<SocketAddr>()?;
    let listener = TcpListener::bind(&addr).await?;
 
    let json_parser = ExampleJSONParser::new();
    let parsers: Vec<Arc<Mutex<Box<MessageProtocolParser+Send>>>>  = vec![Arc::new(Mutex::new(Box::new(json_parser)))];

    /*
    loop  {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            loop {
                let (r, w)  = socket.split();
                let mut framed_writer = FramedWrite::new(w, MyBytesCodec::new(parsers));
                let mut framed_reader = FramedRead::new(r, MyBytesCodec::new(parsers));
                //framed_reader.next().map(|e| e.unwrap()).await?;
            }
        
        });
    }
    */
    Ok(())
}
