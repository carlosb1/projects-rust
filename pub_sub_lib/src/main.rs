extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;
extern crate pub_sub;


use tokio_util::codec::{FramedWrite, FramedRead};
use tokio::net::TcpListener;
use std::net::SocketAddr;
use std::error::Error;
use futures::{SinkExt, StreamExt};
use pub_sub::{MyBytesCodec, JSONParser};



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
