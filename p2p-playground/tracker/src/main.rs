mod devices;
mod domain;
mod entrypoint;
mod services;

extern crate serde_derive;

use crate::entrypoint::Server;
use crate::services::MessageManager;
use futures::lock::Mutex;
use log::info;
use std::sync::Arc;

use tokio::runtime::Runtime;

pub fn main() {
    pretty_env_logger::init();
    let rt = Runtime::new().unwrap();

    let user = "user".to_string();
    let address = "127.0.0.1:12345".to_string();

    let message_manager = MessageManager::new();
    let message_manager: Arc<Mutex<MessageManager>> = Arc::new(Mutex::new(message_manager));
    let server = Server {};
    info!("Running our server");
    let _ = rt.block_on(server.run(address, user, message_manager));
}
