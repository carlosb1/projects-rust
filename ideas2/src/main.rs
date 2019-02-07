#[macro_use]
extern crate serde_derive;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write, Error};

pub struct MsgManager;

impl MsgManager {
    fn run(&self, message: Message) {
        match message.operation.as_ref() {
            "run" => println!("Running!!"),
            _ => println!("Nothing to do"),
        };
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    operation: String,
}

impl Message {
    fn new(operation: String) -> Message {
        Message {operation: operation} 
    }
    fn new_empty() -> Message  {
        Message {operation: "".to_string()}
    }
    
}



fn handle_client(mut stream: TcpStream) -> Result<(),Error> {
    println!("Incoming connection from: {}",stream.peer_addr()?);

    let mut vec = Vec::new();
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf)?;
         if bytes_read == 0 {
            println!("vect = {:?}", vec);
            return Ok(());
         }
         vec.extend(buf[..bytes_read].iter().cloned());
         let str_vec = String::from_utf8(vec.to_owned()).expect("Found invalid UTF-8");
         
         if str_vec.contains("</end>") {
            let res_vec = str_vec.as_str().split("</end>").collect::<Vec<&str>>()[0];
            //println!("vect = {:}", str_vec.split("</end>")[0].to_string());
            println!("message = {:?}", res_vec);
            println!("FINISHED!!"); 
            return Ok(());
         }
      stream.write(&buf[..bytes_read])?;
    }
}

fn main () {
    let listener = TcpListener::bind("0.0.0.0:8888").expect("Could not bind");
    for stream in listener.incoming() {
        match stream {
            Err(e) => { eprintln!("failed: {}",e)}
            Ok(stream) => {
                thread::spawn( move || {
                    handle_client(stream).unwrap_or_else(|error| eprintln!("{:?}",error));
                });
            }
        }
    }
}
