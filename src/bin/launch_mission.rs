use async_std::io;
use futures::{prelude::*, select};
use libp2p::gossipsub::MessageId;
use libp2p::gossipsub::{
    GossipsubEvent, GossipsubMessage, IdentTopic as Topic, MessageAuthenticity, ValidationMode,
};
use libp2p::{gossipsub, identity, swarm::SwarmEvent, Multiaddr, PeerId};
use std::collections::hash_map::DefaultHasher;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::time::Duration;
#[macro_use(array)]
extern crate ndarray;
use serde_json;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Gossipsub topic
    let topic = Topic::new("topic");

    let mut swarm = { // Build and implicitly return swarm

        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &GossipsubMessage| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
            .duplicate_cache_time(Duration::from_secs(1))
            .build()
            .expect("Valid config");

        // build a gossipsub network behaviour
        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
                .expect("Correct configuration");

        // subscribes to our topic
        gossipsub.subscribe(&topic).unwrap();

        // build the swarm
        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    let address: Multiaddr = "/ip4/127.0.0.1/tcp/60740".parse().unwrap();
    match swarm.dial(address.clone()) {
        Ok(_) => {
            println!("Dialed {:?}", address);

            // Poll the swarm until a connection was established or the dial failed.
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::ConnectionEstablished {..} => {
                        println!("Connection established");
                        break
                    },
                    SwarmEvent::OutgoingConnectionError {..} => todo!(), // connection failed
                    _ => {}
                };
                if let Err(e) = swarm
                .behaviour_mut()
                .publish(topic.clone(), "THIS IS A TEST".as_bytes())
            {
                println!("Publish error: {:?}", e);
            };
            }

            if let Err(e) = swarm
                .behaviour_mut()
                .publish(topic.clone(), "THIS IS A TEST".as_bytes())
            {
                println!("Publish error: {:?}", e);
            };
        },
        Err(e) => println!("Dial {:?} failed: {:?}", address, e),
    };

    let mut stdin = io::BufReader::new(io::stdin()).lines().fuse();

    loop {
        select! {
            line = stdin.select_next_some() => {

                // Create initial area and publish it to mothership.
                // This shouldn't have to take place within stdin events but there is an issue publishing elsewhere in code.

                let mission_area: ndarray::Array2<u32> = array![[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                                                                [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]];

                let serialized = serde_json::to_string(&mission_area).unwrap();

                if let Err(e) = swarm
                    .behaviour_mut()
                    .publish(topic.clone(), serialized.as_bytes())
                {
                    println!("Publish error: {:?}", e);
                };
            },
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(GossipsubEvent::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                }) => println!(
                    "Got message: {} with id: {} from peer: {:?}",
                    String::from_utf8_lossy(&message.data),
                    id,
                    peer_id
                ),
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                _ => {}
            }
        }
    }

}