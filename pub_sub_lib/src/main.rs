use std::{thread, time};
use log::info;
use pub_sub::Manager;

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
