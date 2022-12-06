use crate::devices::JSONMessage;
use crate::services::MessageManager;
use bytes::BytesMut;
use futures::lock::Mutex;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_util::codec::{Decoder, Encoder};
use tokio_util::codec::{FramedRead, FramedWrite};

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

                            let mut _manager = _manager.lock().await;
                            if let Some(resp) = _manager.exec(str_message.clone()) {
                                let _ = framed_writer.send(resp.to_json().unwrap().into_bytes());
                                //TODO it is not necessary to verify the response
                            }
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
