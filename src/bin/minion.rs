use std::collections::VecDeque;
use std::error::Error;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use futures::{prelude::*, select};
use libp2p::{gossipsub, identity, swarm::SwarmEvent, Multiaddr, PeerId};
use libp2p::gossipsub::{GossipsubEvent, IdentTopic as Topic, MessageAuthenticity, ValidationMode};

use async_std::stream;

use libd2d::{ DelegateTaskMessage, MinionState, Coordinate, ActorPosition, Minion, minion_bot };


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // Set initial state
    let mut state = MinionState {
        position: ActorPosition {
            coordinates: Coordinate { x: -5, y: -5 },
            orientation: 0.
        },
        tasks: Arc::new(Mutex::new(VecDeque::new())),
        points_of_interest: Arc::new(Mutex::new(VecDeque::new())),
        // waker: Arc::new(Mutex::new(None))
    };

    // create robot thread
    let tasks = Arc::clone(&state.tasks);
    let pois = Arc::clone(&state.points_of_interest);
    let _ = thread::spawn(move || minion_bot(tasks, pois));

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    let topic_delegate_task = Topic::new("delegate_task");
    let topic_heartbeat = Topic::new("heartbeat");

    // Create a Swarm to manage peers and events
    let mut swarm = {

        let gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
            .validation_mode(ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message signing)
            .build()
            .expect("Valid config");

        // build a gossipsub network behaviour
        let mut gossipsub: gossipsub::Gossipsub =
            gossipsub::Gossipsub::new(MessageAuthenticity::Signed(local_key), gossipsub_config)
                .expect("Correct configuration");

        gossipsub.subscribe(&topic_delegate_task).unwrap();
        gossipsub.subscribe(&topic_heartbeat).unwrap();

        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    let address: Multiaddr = "/ip4/127.0.0.1/tcp/60740".parse().unwrap();
    match swarm.dial(address.clone()) {
        Ok(_) => println!("Dialed {:?}", address),
        Err(e) => println!("Dial {:?} failed: {:?}", address, e),
    };

    let mut heartbeat_interval = stream::interval(Duration::from_secs(4)).fuse();

    // Stream here needs to 
    let poi_stream = (&mut state).fuse();

    loop {
        select! {
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(GossipsubEvent::Message {
                    propagation_source: _peer_id,
                    message_id: _id,
                    message,
                }) => {
                    match message.topic.as_str() {

                        "delegate_task" => {

                            let task: DelegateTaskMessage = 
                                serde_json::from_str(
                                    &String::from_utf8_lossy(&message.data)
                                ).unwrap();

                            if task.peer_id == local_peer_id {
                            { // update state
                                let mut tasks = state.tasks.lock().unwrap();
                                tasks.push_back(task.area);
                            }
                            println!("{:?}", state);

                                // Commence search

                            };

                        },

                        _ => {}
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                _ => {}
            },

            _ = heartbeat_interval.select_next_some() => {
                let heartbeat = Minion {
                    peer_id: local_peer_id,
                    position: state.position.coordinates.clone()
                };

                let heartbeat = serde_json::to_string(&heartbeat).unwrap();

                if let Err(e) = swarm
                    .behaviour_mut()
                    .publish(topic_heartbeat.clone(), heartbeat.as_bytes())
                {
                    println!("Publish error: {:?}", e);
                };

            }
        }
    }
}