extern crate serde_derive;

extern crate bytes;
extern crate tokio;
extern crate serde;
extern crate serde_json;
extern crate pub_sub;
extern crate pretty_env_logger;
extern crate log;

pub mod manager;

use std::{thread, time};
use log::info;
use manager::Manager;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init(); 
    let user = "user".to_string();
    let address = "127.0.0.1:12345".to_string();
    let filepath_db = "infodb".to_string();
    let manager = Manager::new(filepath_db, user, address);
    manager.init();
    info!("It was initialized");
    let sec_times = time::Duration::from_secs(60);
    thread::sleep(sec_times);
    Ok(())
}
