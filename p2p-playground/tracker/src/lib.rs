mod client;
mod message;

#[macro_use]
extern crate serde_derive;

use crate::message::{JSONMessage, Message};
use bytes::BytesMut;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_util::codec::{Decoder, Encoder};
use tokio_util::codec::{FramedRead, FramedWrite};

// - blockchain validate library
// - IPFS for searching storage
// - generate CIDs
// - generate tracker
//      - return a list of nodes, its IPS, mdns?

/// Byte encoder / decoder for Tokio.
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

impl Encoder<Vec<u8>> for MyBytesCodec {
    type Error = io::Error;

    fn encode(&mut self, data: Vec<u8>, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(data);
        Ok(())
    }
}
/// Trait for replies. it includes trigger functions for each type of message.
pub trait MessageReplier: Send + Sync {
    fn no_oper(self: Box<Self>, messg: &Message);
    fn box_clone(&self) -> Box<dyn MessageReplier>;
}

/// Dispatcher class for each type of responses.
pub struct MessageManager {
    replier: Box<dyn MessageReplier>,
}

impl MessageManager {
    fn new(replier: Box<dyn MessageReplier>) -> MessageManager {
        MessageManager { replier }
    }
    fn exec(self, str_messg: String) -> Option<Box<Message>> {
        let messg: Message =
            serde_json::from_str(&str_messg).expect("It was not parsed json message to string");
        let oper = messg.operation.as_str();
        match oper {
            _ => {
                self.replier.no_oper(&messg);
                None
            }
        }
    }
}

/// Server TCP implementation for tokio.
#[derive(Clone)]
pub struct Server;

impl Server {
    pub async fn run(
        self,
        address: String,
        user: String,
        replier: Arc<Mutex<Box<dyn MessageReplier>>>,
    ) -> Result<(), Box<dyn Error>> {
        info!("Trying to connect to {}", address);

        let addr = address.as_str().parse::<SocketAddr>()?;

        let listener = TcpListener::bind(&addr).await?;
        loop {
            let replier = replier.clone();
            let user = user.clone();
            let address = address.clone();

            info!("Wait for a new socket...");
            let (mut socket, _) = listener.accept().await?;
            tokio::spawn(async move {
                let (r, w) = socket.split();
                let mut framed_writer = FramedWrite::new(w, MyBytesCodec {});
                let mut framed_reader = FramedRead::new(r, MyBytesCodec {});

                if let Some(frame) = framed_reader.next().await {
                    match frame {
                        Ok(message) => {
                            let mut response_message = Box::new(Message::new_user(user, address));
                            let _ = {
                                let _repl = replier
                                    .lock()
                                    .expect("It was not possible to unlock shared replier message");
                                let _manager = MessageManager::new((*_repl).box_clone());
                                let str_message = String::from_utf8(message)
                                    .expect("It was not possible to parse message to a string");
                                match _manager.exec(str_message) {
                                    Some(response) => response_message = response,
                                    None => {
                                        info!("It is not necessary to reply the message")
                                    }
                                };
                            };
                            framed_writer
                                .send(
                                    response_message
                                        .to_json()
                                        .expect("Error parsing json message")
                                        .as_bytes()
                                        .to_vec(),
                                )
                                .await
                                .map_err(|e| println!("not response! {}", e))
                                .ok();
                        }
                        Err(e) => {
                            error!("Error received while we are reading {}", e);
                        }
                    }
                }
            });
        }
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            let mut buf = [0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = match socket.read(&mut buf).await {
                    // socket closed
                    Ok(n) if n == 0 => return,
                    Ok(n) => n,
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Write the data back
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
}
