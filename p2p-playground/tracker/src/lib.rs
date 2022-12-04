mod application;
mod client;
mod message;

#[macro_use]
extern crate serde_derive;

use crate::message::{JSONMessage, Message};
use bytes::BytesMut;
use dyn_clone::DynClone;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
//use futures::lock::Mutex;
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
pub trait MessageReplier: DynClone {
    fn run(self: Box<Self>, messg: Message) -> Option<Message>;
}

dyn_clone::clone_trait_object!(MessageReplier);

/// Dispatcher class for each type of responses.
#[derive(Clone)]
pub struct MessageManager {
    replier: HashMap<String, Box<dyn MessageReplier>>,
}

impl MessageManager {
    fn new() -> MessageManager {
        MessageManager {
            replier: HashMap::new(),
        }
    }
    fn exec(&mut self, str_messg: String) -> Option<Message> {
        let messg: Message =
            serde_json::from_str(&str_messg).expect("It was not parsed json message to string");
        let oper = messg.operation.as_str();
        self.replier
            .get(oper)
            .map_or(None, |repl| repl.clone().run(messg))
    }
}
unsafe impl Send for MessageManager {}
unsafe impl Sync for MessageManager {}

/// Server TCP implementation for tokio.
#[derive(Clone)]
pub struct Server;

impl Server {
    pub async fn run(
        self,
        address: String,
        _user: String,
        manager: Arc<Mutex<Box<MessageManager>>>,
    ) -> Result<(), Box<dyn Error>> {
        info!("Trying to connect to {}", address);

        let addr = address.as_str().parse::<SocketAddr>()?;
        let listener = TcpListener::bind(&addr).await?;
        loop {
            let _manager = manager.clone();
            info!("Wait for a new socket...");
            let (mut socket, _) = listener.accept().await?;
            tokio::spawn(async move {
                let (r, w) = socket.split();
                let mut framed_writer = FramedWrite::new(w, MyBytesCodec {});
                let mut framed_reader = FramedRead::new(r, MyBytesCodec {});

                if let Some(frame) = framed_reader.next().await {
                    match frame {
                        Ok(message) => {
                            let str_message = String::from_utf8(message)
                                .expect("It was not possible to parse message to a string");

                            let mut _manager = _manager.lock().unwrap();
                            let resp = _manager.exec(str_message.clone());
                            let unwrapped_resp =
                                resp.unwrap_or_default().to_json().unwrap().into_bytes();
                            let _ = framed_writer.send(unwrapped_resp);
                            //TODO it is not necessary to verify the response
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
