
use libp2p::kad::record::store::MemoryStore;
use libp2p::kad::{Kademlia, KademliaEvent, PutRecordOk, Record, Quorum};
use libp2p::kad::record::Key;
use libp2p::{
    Swarm,
    NetworkBehaviour,
    mdns::{Mdns, MdnsEvent},
    swarm::NetworkBehaviourEventProcess
};

// We create a custom network behaviour that combines Kademlia and mDNS.
#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub kademlia: Kademlia<MemoryStore>,
    pub mdns: Mdns
}
impl NetworkBehaviourEventProcess<MdnsEvent> for MyBehaviour {
      // Called when `mdns` produces an event.
      fn inject_event(&mut self, event: MdnsEvent) {
          if let MdnsEvent::Discovered(list) = event {
              for (peer_id, multiaddr) in list {
                  self.kademlia.add_address(&peer_id, multiaddr);
              }
          }
      }
  }

impl NetworkBehaviourEventProcess<KademliaEvent> for MyBehaviour {
    // Called when `kademlia` produces an event.
    fn inject_event(&mut self, message: KademliaEvent) {
        match message {
            KademliaEvent::GetRecordResult(Ok(result)) => {
                for Record { key, value, .. } in result.records {
                    println!(
                        "Got record {:?} {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap(),
                        std::str::from_utf8(&value).unwrap(),
                    );
                }
            }
            KademliaEvent::GetRecordResult(Err(err)) => {
                eprintln!("Failed to get record: {:?}", err);
            }
            KademliaEvent::PutRecordResult(Ok(PutRecordOk { key })) => {
                println!(
                    "Successfully put record {:?}",
                    std::str::from_utf8(key.as_ref()).unwrap()
                );
            }
            KademliaEvent::PutRecordResult(Err(err)) => {
                eprintln!("Failed to put record: {:?}", err);
            }
            _ => {}
        }
    }
}
pub struct Client<'a> {
    swarm: &'a mut Swarm<MyBehaviour>,
}

impl<'a> Client<'a>{
    fn search_by_username(self){
        let key=Key::new(& "hello world".to_string());
        let value = vec![1, 2, 3, 4];

        let record = Record {
            key,
            value,
            publisher: None,
            expires: None,
        };
        self.swarm.kademlia.put_record(record, Quorum::One);
    }
}
