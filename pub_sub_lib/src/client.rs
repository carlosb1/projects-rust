use pub_sub::{JSONMessage, Message, send};
use tokio::runtime::Runtime;
use std::env;
use std::error::Error;


pub fn main () -> Result<(), Box<dyn std::error::Error>>{
    let args: Vec<String> = env::args().collect();
    let mut address = "127.0.0.1:12345".to_string();
    let user = "user".to_string();
    if args.len() >= 2 {
        address = args[0].clone();   
    }

    let mut rt = Runtime::new()?;
    
    println!("Testing ack");
    let message  = Message::ack(user.clone(), address.clone());
    let _ = rt.block_on(send(address.clone().as_str(), message.to_json().expect("Error parsing json message").as_str()));

    println!("Testing login");
    let message  = Message::subscribe("topic1".to_string(),"me".to_string(), "192.168.0.1".to_string());
    let result:  Result<Box<Message>, Box<dyn Error>>  = rt.block_on(send(address.as_str(), message.to_json().expect("Error parsing json message").as_str()));
    
    match result {
        Ok(message) =>{
            println!("{}",message.to_json().expect("Error parsing json message").as_str());
        },
        Err(e) => {
            println!("{}?", e)
        },
    }

    Ok(())
}

