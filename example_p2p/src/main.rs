#[macro_use]
extern crate serde_derive;

extern crate tokio;
extern crate tokio_codec;


use tokio::codec::Decoder;
use tokio_codec::BytesCodec;
use tokio::prelude::*;
use tokio::net::TcpListener;

extern crate serde;
extern crate serde_json;


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
