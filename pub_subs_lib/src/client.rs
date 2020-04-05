

//type ClientTransport = Framed<TcpStream, MyBytesCodec>;
use tokio_util::codec::{FramedWrite, FramedRead};
use tokio::net::TcpStream;
use std::net::SocketAddr;
use std::error::Error;
use futures::{SinkExt, StreamExt};
use pub_sub::{MyBytesCodec, JSONParser};
use futures::executor::block_on;
use std::env;


pub async fn run(address: String) -> Result<(), Box<dyn Error>> {
    println!("Trying to connect to {}", address);
    let remote_address: SocketAddr = address.parse().unwrap();
    let mut tcp = TcpStream::connect(&remote_address).await?;
    let (r, w) = tcp.split();
    
    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
    let mut framed_reader = FramedRead::new(r, MyBytesCodec{});
    
    let encoded: Vec<u8> = vec![1,1];
    framed_writer.send(encoded).await?;

    if let Some(frame) = framed_reader.next().await {
        match frame {
            Ok(message) => {
                let json_parser = JSONParser::new();
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

    Ok(()) 
}

pub fn main () {
    let args: Vec<String> = env::args().collect();
    let mut address = "127.0.0.1:12345".to_string();
    if args.len() >= 2 {
        address = args[0].clone();   
    }
    let future = run(address);
    block_on(future);
}

