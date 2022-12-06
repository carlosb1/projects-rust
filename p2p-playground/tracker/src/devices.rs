use crate::domain::Message;
use crate::entrypoint::MyBytesCodec;
use futures::{SinkExt, StreamExt};
use log::{error, info};
use std::error::Error;
use std::io::ErrorKind;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};

impl JSONMessage for Message {
    fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
    fn get_operation(self) -> String {
        self.operation
    }
}

//// Trait for JSON message. Function contracts for serialize messages.
pub trait JSONMessage {
    fn to_json(&self) -> Result<String, serde_json::Error>;
    fn get_operation(self) -> String;
}

/// Send function for tokio. It sends json messages.
pub async fn send(address: &str, mesg: &str) -> Result<Box<Message>, Box<dyn Error>> {
    info!("Trying to connect to {}", address);
    let remote_address: SocketAddr = address
        .parse()
        .expect("it was not possible to parse net address");
    let mut tcp = TcpStream::connect(&remote_address).await?;
    let (r, w) = tcp.split();

    let mut framed_writer = FramedWrite::new(w, MyBytesCodec {});
    let mut framed_reader = FramedRead::new(r, MyBytesCodec {});

    let encoded: Vec<u8> = mesg.as_bytes().to_vec();
    framed_writer.send(encoded).await?;
    info!("It received a response");

    if let Some(frame) = framed_reader.next().await {
        match frame {
            Ok(response) => {
                let str_messg = String::from_utf8(response)
                    .expect("It was not possible to parse message to a string");
                info!("{:?}", str_messg);
                let messg: Message = serde_json::from_str(&str_messg)
                    .expect("It was not parsed json message to Message");
                return Ok(Box::new(messg));
            }
            Err(e) => {
                error!("Error received while we are reading {}", e);
                return Err(Box::new(e));
            }
        }
    } else {
        return Err(Box::new(std::io::Error::new(
            ErrorKind::Other,
            "It was no possible to receive response from server",
        )));
    }
}
