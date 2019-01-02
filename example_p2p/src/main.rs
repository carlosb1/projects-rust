extern crate tokio;
extern crate tokio_codec;

use tokio_codec::BytesCodec;
use tokio::prelude::*;
use tokio::net::TcpListener;
use tokio::codec::Decoder;

fn main() {
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

    let server = listener.incoming()
            .map_err(|e| eprintln!("accept failed = {:?}", e))
            .for_each(move |socket| {
                let framed = BytesCodec::new().framed(socket);
                let (_writer, reader) = framed.split();

                let handle_conn = reader.for_each(|bytes| {
                    println!("bytes: {:?}", bytes);
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
