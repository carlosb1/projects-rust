

//type ClientTransport = Framed<TcpStream, MyBytesCodec>;
use pub_sub::{Message, send};
use futures::executor::block_on;
use std::env;


pub fn main () {
    let args: Vec<String> = env::args().collect();
    let mut address = "127.0.0.1:12345".to_string();
    if args.len() >= 2 {
        address = args[0].clone();   
    }
    let future = send(address, Message::new("first_oper".to_string()));
    block_on(future);
}

