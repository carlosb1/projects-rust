#[macro_use]
extern crate serde_derive;

extern crate tokio;
extern crate tokio_codec;


use tokio::codec::Decoder;
use tokio_codec::BytesCodec;
use tokio::prelude::*;
use tokio::net::TcpListener;
use tokio::io::*;
use std::fs::File;
use std::io::prelude::*;
use chrono::prelude::*;
use chrono::offset::LocalResult;

extern crate serde;
extern crate serde_json;

pub struct MsgManager;


impl MsgManager {
    fn run(&self, message: Message) {
        match message.operation.as_ref() {
            "run" => println!("Running!!"),
            "upload" => println!("Uploading!!"),
            "finish" => println!("Finishing!!"),
            _ => println!("Nothing to do"),
        };
    }

    fn upload(&self) {
       let addr = "127.0.0.1:12346".parse().unwrap();
       let socket = TcpListener::bind(&addr).expect("Unable to bind TCP listener");
       let done = socket.incoming()
           .map_err(|e| eprintln!("Failed to accept socket = {:?}", e))
           .for_each(move |socket| {
            let framed = BytesCodec::new().framed(socket);
            let (_writer, reader) = framed.split();
            let handle_conn = reader.for_each(|bytes| {
                    
                    let utc: DateTime<Utc> = Utc::now();
                    let mut string_utc: String = utc.to_rfc3339();
                    string_utc.push_str("bin");
                    let v: Vec<u8> =  bytes.iter().map(|b| *b).collect();
                    let mut file_buf = File::create(string_utc)?;
                    file_buf.write_all(&v)?;
                    Ok(())
            });
            tokio::spawn(handle_conn.map_err(|e| Err(e).unwrap()))
            });
       tokio::run(done);
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
    

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

    let server = listener.incoming()
            .map_err(|e| eprintln!("accept failed = {:?}", e))
            .for_each(move |socket| {
                let framed = BytesCodec::new().framed(socket);
                let (_writer, reader) = framed.split();
                let handle_conn = reader.for_each(|bytes| {
                    //println!("bytes: {:?}", bytes);
                    let v: Vec<u8> =  bytes.iter().map(|b| *b).collect();
                    let message = String::from_utf8(v).unwrap();
                    println!("in text: {}", message);
                    let msg: Message = match serde_json::from_str(&message)  {
                            Err(..) =>   {println!("It was not parsed correctly"); Message::new_empty() },
                            Ok(msg) => msg,
                    };  
                    Ok(())
                })
                .and_then(|()| {
                    println!("Socket received FIN packet and closed connection");
                    Ok(())
                })
                .or_else(|err| {
                    println!("Socked closed with error: {:?}", err);
                    Err(err)

                })
                .then(|result| {
                    println!("Socket closed with result: {:?}", result);
                    Ok(())
                });
               
            tokio::spawn(handle_conn)
    });
    tokio::run(server);
}
