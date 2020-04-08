

//type ClientTransport = Framed<TcpStream, MyBytesCodec>;
use pub_sub::{FactoryMessage,JSONMessage, Message,  send};
use futures::executor::block_on;
use std::env;


pub fn main () {
    let args: Vec<String> = env::args().collect();
    let mut address = "127.0.0.1:12345".to_string();
    if args.len() >= 2 {
        address = args[0].clone();   
    }
    let message  = FactoryMessage::ack();
    let future = send(address, message.to_json().unwrap());
    block_on(future);
}

