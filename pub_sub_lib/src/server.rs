extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;
extern crate pub_sub;


use tokio::runtime::Runtime;
use pub_sub::{Server};




fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rt = Runtime::new()?;
    let server = Server{};
    rt.block_on(server.run("127.0.0.1:12345".to_string()))

}
