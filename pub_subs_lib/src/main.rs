#[macro_use]
extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;

use bytes::BytesMut;
use std::io;
use tokio_util::codec::{Encoder,Decoder, FramedWrite, FramedRead};
use tokio::net::{TcpListener, TcpStream};
use std::net::SocketAddr;
use std::error::Error;
use futures::{SinkExt, StreamExt};



#[derive(Clone,Copy)]
pub struct ExampleJSONParser;
impl ExampleJSONParser {
    fn new() -> ExampleJSONParser {
        ExampleJSONParser{}
    }
}

impl ExampleJSONParser {
    fn parse(&self, info: &Vec<u8>) -> Option<Message> {
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
   
//type ClientTransport = Framed<TcpStream, MyBytesCodec>;

pub async fn run(address: String) -> Result<(), Box<dyn Error>> {

    let remote_address: SocketAddr = address.parse().unwrap();
    let mut tcp = TcpStream::connect(&remote_address).await?;
    let (_, w) = tcp.split();
    
    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
    
    //let mut framed_reader = FramedRead::new(r, MyBytesCodec::new(parsers));
    let encoded: Vec<u8> = vec![1,1];
    framed_writer.send(encoded).await?;
    Ok(()) 
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:12345".parse::<SocketAddr>()?;
    let mut listener = TcpListener::bind(&addr).await?;
 
    loop  {
            println!("Wait for a new socket...");
            let (mut socket, _) = listener.accept().await?;
            tokio::spawn(async move {
            loop {
                println!("Running new thread connection");
                let (r, w)  = socket.split();
                let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
                let mut framed_reader = FramedRead::new(r, MyBytesCodec{});

                if let Some(frame) = framed_reader.next().await {
                    match frame {
                        Ok(message) => {
                            let json_parser = ExampleJSONParser::new();
                            println!("{:?}", message);
                            json_parser.parse(&message);
                            let resp: Vec<u8>  = vec![1,2];
                            framed_writer.send(resp).await.map_err(|e| println!("not response! {}", e)).ok();
                        }
                        Err(e) => {
                            println!("Error received while we are reading {}", e);
                        }

                    }
                }

            }
        
        });
    }
}
