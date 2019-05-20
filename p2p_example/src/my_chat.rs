use futures::prelude::*;
use std::sync::{Arc, Mutex};

use libp2p::{
    PeerId,
    Swarm,
    identity,
    NetworkBehaviour,
    tokio_codec::{FramedRead, LinesCodec}
};

#[derive(Clone)]
struct NodeManager {
    pub addresses: Vec<String>,
}
impl NodeManager {
    fn new() -> NodeManager {
        NodeManager{addresses: Vec::new()}
    }
    fn say_hello(self) {
        println!("Hello to: {:?}",self.addresses);   
    }
}

#[derive(NetworkBehaviour)]
struct MyBehaviour<TSubstream: libp2p::tokio_io::AsyncRead + libp2p::tokio_io::AsyncWrite> {
    floodsub: libp2p::floodsub::Floodsub<TSubstream>,
    mdns: libp2p::mdns::Mdns<TSubstream>,
    #[behaviour(ignore)]
    node_manager: Arc<Mutex<NodeManager>>,
}


impl<TSubstream: libp2p::tokio_io::AsyncRead + libp2p::tokio_io::AsyncWrite> libp2p::core::swarm::NetworkBehaviourEventProcess<libp2p::mdns::MdnsEvent> for MyBehaviour<TSubstream> {
    fn inject_event(&mut self, event: libp2p::mdns::MdnsEvent) {
        match event {
            libp2p::mdns::MdnsEvent::Discovered(list) => {
                for (peer, _) in list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            },
            libp2p::mdns::MdnsEvent::Expired(list) => {
                for (peer, _) in list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        } 
    }

}


impl<TSubstream: libp2p::tokio_io::AsyncRead + libp2p::tokio_io::AsyncWrite> libp2p::core::swarm::NetworkBehaviourEventProcess<libp2p::floodsub::FloodsubEvent> for MyBehaviour<TSubstream> {
    fn inject_event(&mut self, message: libp2p::floodsub::FloodsubEvent) {
        if let libp2p::floodsub::FloodsubEvent::Message(message) = message {
            println!("Received '{:?}' from {:?}", String::from_utf8_lossy(&message.data), message.source);
            println!("---> my bytes {:?}", message.source.to_base58());
            if !self.node_manager.lock().unwrap().addresses.contains(&message.source.to_base58()) {
                (*self.node_manager).lock().unwrap().addresses.push(message.source.to_base58());
            }
        }
    }
}




// sh cargo run 
// sh run /ip4/127.0.0.1/tcp/24915
fn main() {
    env_logger::init();
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());

    let transport = libp2p::build_development_transport(local_key);

    let floodsub_topic = libp2p::floodsub::TopicBuilder::new("chat").build();

    let  node_manager: NodeManager =  NodeManager::new();
    let rc_node = Arc::new(Mutex::new(node_manager));

    // lambda function to initialise values
    let mut swarm =  {
        let mut behaviour = MyBehaviour {
            floodsub: libp2p::floodsub::Floodsub::new(local_peer_id.clone()),
            mdns: libp2p::mdns::Mdns::new().expect("Failed to create mDNS service"),
            node_manager: rc_node.clone(),
        };
        behaviour.floodsub.subscribe(floodsub_topic.clone());
        libp2p::Swarm::new(transport, behaviour, local_peer_id)

    };


    let addr = libp2p::Swarm::listen_on(&mut swarm, "/ip4/0.0.0.0/tcp/0".parse().unwrap()).unwrap();
    println!("Listening on {:?}", addr);

    // get value
    if let Some(to_dial) = std::env::args().nth(1) {
        let dialing = to_dial.clone();
        match to_dial.parse() {
            Ok(to_dial) => {
                match libp2p::Swarm::dial_addr(&mut swarm, to_dial) {
                    Ok(_) => println!("Dialed {:?}", dialing),
                    Err(e) => println!("Dial {:?} failed: {:?}", dialing, e)
                }
            }, Err(err) => println!("Failed to parse address to dial {:?}", err)
        }
    }

    let stdin = tokio_stdin_stdout::stdin(0);
    let mut framed_stdin = FramedRead::new(stdin, LinesCodec::new());

    tokio::run(futures::future::poll_fn(move || -> Result<_, ()> {
            loop {
                match framed_stdin.poll().expect("Error while polling stdin") {
                    Async::Ready(Some(line)) => {
                        (*rc_node).lock().unwrap().clone().say_hello();
                        swarm.floodsub.publish(&floodsub_topic, line.as_bytes())
                    },
                    Async::Ready(None) => panic!("Stdin closed"),
                    Async::NotReady => break,
                };
            }
            loop {
                match swarm.poll().expect("Error while polling swarm") {
                    Async::Ready(Some(_)) => {
                    },
                    Async::Ready(None) | Async::NotReady =>  break,
                }
            }
            Ok(Async::NotReady)
    }));

}
