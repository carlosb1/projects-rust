extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;
extern crate pub_sub;


use tokio_util::codec::{FramedWrite, FramedRead};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use std::net::SocketAddr;
use std::error::Error;
use futures::{SinkExt, StreamExt};
use pub_sub::{MyBytesCodec, JSONParser};


pub struct Server;

impl Server {
    async fn run(self, address: String) -> Result<(), Box<dyn Error>> { 
        println!("Trying to connect to {}", address);
        let addr = address.as_str().parse::<SocketAddr>()?;
        let mut listener = TcpListener::bind(&addr).await?; 
            loop  {
                println!("Wait for a new socket...");
                let (mut socket, _) = listener.accept().await?;
                tokio::spawn(async move {
                loop {
                    let (r, w)  = socket.split();
                    let mut framed_writer = FramedWrite::new(w, MyBytesCodec{});
                    let mut framed_reader = FramedRead::new(r, MyBytesCodec{});
    
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

                }
            });
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rt = Runtime::new()?;
    let server = Server{};
    rt.block_on(server.run("127.0.0.1:12345".to_string()))

}
