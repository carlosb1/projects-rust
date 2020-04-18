extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;
extern crate pub_sub;


use tokio::runtime::Runtime;
use pub_sub::{Server, MessageReplier, MockReplier};
use std::sync::{Arc, Mutex};



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut rt = Runtime::new()?;
    
    let user = "user".to_string();
    let address = "127.0.0.1:12345".to_string();

    let mock_replier = MockReplier::new(user.clone(), address.clone());
    let replier: Arc<Mutex<Box<dyn MessageReplier>>> = Arc::new(Mutex::new(Box::new(mock_replier)));
    
    //mock_replier.subscriptions.clone().get("user");

    let server = Server{};
    rt.block_on(server.run(user, address,replier))

}
