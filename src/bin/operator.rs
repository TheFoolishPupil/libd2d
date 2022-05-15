use futures::{prelude::*, select};
use libd2d::Coordinate;
use libp2p::gossipsub::{GossipsubEvent, IdentTopic as Topic, MessageAuthenticity, ValidationMode};
use libp2p::{gossipsub, identity, swarm::SwarmEvent, Multiaddr, PeerId};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;
// extern crate ndarray;
use serde_json;
use ndarray::Array;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::Uniform;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Gossipsub topic
    let topic_new_mission = Topic::new("new_mission");
    let topic_discovery = Topic::new("discovery");
    let topic_report = Topic::new("reporting");
    let topic_report_mothership = Topic::new("reporting_mothership");
    let topic_mission_complete = Topic::new("mission_complete");

    let mut swarm = {
        // Set a custom gossipsub
        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .duplicate_cache_time(Duration::from_secs(1))
            .build()
            .expect("Valid config");

        // build a gossipsub network behaviour
        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
                .expect("Correct configuration");

        gossipsub.subscribe(&topic_new_mission).unwrap();
        gossipsub.subscribe(&topic_discovery).unwrap();
        gossipsub.subscribe(&topic_report).unwrap();
        gossipsub.subscribe(&topic_report_mothership).unwrap();
        gossipsub.subscribe(&topic_mission_complete).unwrap();

        // build the swarm
        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    // Dial mothership
    let address: Multiaddr = "/ip4/127.0.0.1/tcp/60740".parse().unwrap();
    match swarm.dial(address.clone()) {
        Ok(_) => println!("Dialed {:?}", address),
        Err(e) => println!("Dial {:?} failed: {:?}", address, e),
    };

    // Hardcode local addresses of minions. This is needed because the ports cannot be shared on the same machine.
    let minion_addresses = [
        "/ip4/127.0.0.1/tcp/60741",
        "/ip4/127.0.0.1/tcp/60742",
        "/ip4/127.0.0.1/tcp/60743",
        "/ip4/127.0.0.1/tcp/60744",
        "/ip4/127.0.0.1/tcp/60745",
        "/ip4/127.0.0.1/tcp/60746",
    ];
    
    let mission_area = Array::random((16, 24), Uniform::new(0, 2));
    // let mission_area = Array::random((12, 8), Uniform::new(0, 2));
    // let mission_area = Array::random((53, 67), Uniform::new(0, 2));
    // let mission_area = Array::random((101, 47), Uniform::new(0, 2));
    // let mission_area = Array::random((97, 82), Uniform::new(0, 2));

    let mut result_area = mission_area.clone();

    for cell in result_area.iter_mut() {
        *cell = 1;
    }


    loop {
        select! {

            event = swarm.select_next_some() => match event {

                // Once we know we have subscribers, send the mission
                SwarmEvent::Behaviour(GossipsubEvent::Subscribed {
                    topic: t,
                    ..
                })  if t == topic_new_mission.hash() => {

                    let serialized = serde_json::to_string(&mission_area).unwrap();

                    if let Err(e) = swarm
                        .behaviour_mut()
                        .publish(topic_new_mission.clone(), serialized.as_bytes())
                    {
                        println!("Publish error: {:?}", e);
                    };
                },

                SwarmEvent::Behaviour(GossipsubEvent::Message {
                    propagation_source: _peer_id,
                    message_id: _id,
                    message,
                }) => {
                    match message.topic.as_str() {

                        "discovery" => {

                            let minions: HashMap<PeerId, Coordinate> = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            for (i, (_, _)) in minions.iter().enumerate() {
                                let address: Multiaddr = minion_addresses[i].parse().unwrap();
                                match swarm.dial(address.clone()) {
                                    Ok(_) => println!("Dialed {:?}", address),
                                    Err(e) => println!("Dial {:?} failed: {:?}", address, e),
                                };
                            }
                        }

                        "reporting" => {

                            let minion_coor: (Coordinate, bool) = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            if minion_coor.1 {
                                result_area[[minion_coor.0.x as usize, minion_coor.0.y as usize]] = 2;
                            } else {
                                result_area[[minion_coor.0.x as usize, minion_coor.0.y as usize]] = 0;
                            };
                            println!("\n{}", result_area);
                        },

                        "reporting_mothership" => {
                            let mothership_coor: Coordinate = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            result_area[[mothership_coor.x as usize, mothership_coor.y as usize]] = 1;
                            println!("\n{}", result_area);
                        },

                        "mission_complete" => {
                            assert_eq!(mission_area, result_area);
                            println!("Result area is equal to mission area!");
                        },

                        _ => {}
                    }
                },

                _ => {}
            }
        }
    }
}
