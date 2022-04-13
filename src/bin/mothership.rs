use std::collections::VecDeque;
use std::error::Error;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use futures::{prelude::*, select};
use libp2p::{gossipsub, identity, swarm::SwarmEvent, PeerId};
use libp2p::gossipsub::{GossipsubEvent, IdentTopic as Topic, MessageAuthenticity, ValidationMode};
use serde_json;

use libd2d::{MothershipState, MissionStatus, Coordinate, DelegateTasks, mothership_bot};


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    // Set initial state
    let mut state = MothershipState {
        mission_status: MissionStatus::Pending,
        mission_area: None,
        position: Coordinate { x:0, y:0 },
        tasks: Arc::new(Mutex::new(VecDeque::new())),
        delegate_tasks: DelegateTasks {
            total: 0,
            complete: 0,
        }
    };
    
    { // Aquire lock and create initial tasks
        let mut tasks = state.tasks.lock().unwrap();
        tasks.push_back(Coordinate {x:1, y:1});
        tasks.push_back(Coordinate {x:2, y:2});
        tasks.push_back(Coordinate {x:3, y:3});
    }

    // create robot thread
    let tasks = Arc::clone(&state.tasks);
    let _ = thread::spawn(move || mothership_bot(tasks));


    // Set up comms
    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Gossipsub topic
    let topic = Topic::new("topic");
    let topic_new_mission = Topic::new("new_mission");

    let mut swarm = { // Build and implicitly return swarm

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

        // subscribes to our topic
        gossipsub.subscribe(&topic).unwrap();
        gossipsub.subscribe(&topic_new_mission).unwrap();

        // build the swarm
        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    swarm
        .listen_on("/ip4/127.0.0.1/tcp/60740".parse().unwrap())
        .unwrap();

    loop {
        select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(GossipsubEvent::Message {
                    propagation_source: _peer_id,
                    message_id: _id,
                    message,
                }) => {
                    // println!(
                    //     "Got message: {} with id: {} from peer: {:?}",
                    //     String::from_utf8_lossy(&message.data),
                    //     id,
                    //     peer_id
                    // );
                    match message.topic.as_str() {

                        "new_mission" => {

                            // Update mission state
                            state.mission_status = MissionStatus::InProgress;
                            let serialized_area = String::from_utf8_lossy(&message.data);
                            let area = serde_json::from_str(&serialized_area).unwrap();
                            state.mission_area = Some(area);

                            // Update delegate tasks
                            for peer in swarm.connected_peers() {
                                println!("{:?}", peer);
                            };
                            if let Err(e) = swarm
                                .behaviour_mut()
                                .publish(topic.clone(), "THIS IS A TEST".as_bytes())
                            {
                                println!("Publish error: {:?}", e);
                            }

                        },

                        _ => println!("Unknown topic"),
                    };
                }

                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                _ => {}
            }
        }
    }
}
