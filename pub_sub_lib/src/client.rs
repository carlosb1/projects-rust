use pub_sub::{JSONMessage, Message, send};
use tokio::runtime::Runtime;
use std::env;


pub fn main () -> Result<(), Box<dyn std::error::Error>>{
    let args: Vec<String> = env::args().collect();
    let mut address = "127.0.0.1:12345".to_string();
    if args.len() >= 2 {
        address = args[0].clone();   
    }

    let mut rt = Runtime::new()?;
    let message  = Message::ack();
    let _ = rt.block_on(send(address, message.to_json().unwrap()));
    Ok(())
}

