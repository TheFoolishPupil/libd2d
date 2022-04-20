use std::collections::VecDeque;
use std::error::Error;
use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use futures::{prelude::*, select};
use libp2p::{gossipsub, identity, swarm::SwarmEvent, PeerId};
use libp2p::gossipsub::{GossipsubEvent, IdentTopic as Topic, MessageAuthenticity, ValidationMode};
use serde_json;
use ndarray::Array2;

use libd2d::{MothershipState, MissionStatus, Coordinate, DelegateTasks, DelegateTaskMessage, mothership_bot};


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    // Set initial state
    let mut state = MothershipState {
        mission_status: MissionStatus::Pending,
        mission_area: None,
        position: Coordinate { x:0, y:0 },
        tasks: Arc::new(Mutex::new(VecDeque::new())),
        delegate_tasks: DelegateTasks {
            minions: Vec::new(),
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

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Gossipsub topic
    let topic_discovery = Topic::new("discovery");
    let topic_new_mission = Topic::new("new_mission");
    let topic_delegate_task = Topic::new("delegate_task");

    // Create a Swarm to manage peers and events
    let mut swarm = {

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

        gossipsub.subscribe(&topic_discovery).unwrap();
        gossipsub.subscribe(&topic_new_mission).unwrap();
        gossipsub.subscribe(&topic_delegate_task).unwrap();

        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    swarm
        .listen_on("/ip4/127.0.0.1/tcp/60740".parse().unwrap())
        .unwrap();

    loop {
        select! {
            event = swarm.select_next_some() => match event {

                SwarmEvent::Behaviour(GossipsubEvent::Subscribed {
                    peer_id,
                    topic,
                }) if topic == topic_delegate_task.hash() => {
                    // update delegate tasks according to peers subscribed to topic
                    state.delegate_tasks.minions.push(peer_id);
                    println!("{:?}", state);
                },

                SwarmEvent::Behaviour(GossipsubEvent::Message {
                    propagation_source: peer_id,
                    message_id: _id,
                    message,
                }) => {
                    match message.topic.as_str() {

                        "new_mission" => {

                            // Update state

                            state.mission_status = MissionStatus::InProgress;

                            let serialized_area = String::from_utf8_lossy(&message.data);
                            let area: Array2<u32> = serde_json::from_str(&serialized_area).unwrap();
                            state.mission_area = Some(area.clone());

                            let minion_count = state.delegate_tasks.minions.len();

                            state.delegate_tasks.total = minion_count as u32;
                            println!("{}", minion_count);

                            // TODO: Splitting only exactly for 2 minions. 
                            // This should be generalized for N minions. 
                            // Create a function that returns an iterator?
                            let (dim_x, _) = area.dim();
                            let (subarea_1, subarea_2) = area
                                .view()
                                .split_at(ndarray::Axis(0), dim_x/minion_count);

                            let taskmsg1 = DelegateTaskMessage {
                                peer_id: state.delegate_tasks.minions[0],
                                area: subarea_1.to_owned(),
                            };

                            let taskmsg2 = DelegateTaskMessage {
                                peer_id: state.delegate_tasks.minions[1],
                                area: subarea_2.to_owned(),
                            };

                            let serialized1 = serde_json::to_string(&taskmsg1).unwrap();
                            let serialized2 = serde_json::to_string(&taskmsg1).unwrap();

                            if let Err(e) = swarm
                                .behaviour_mut()
                                .publish(topic_delegate_task.clone(), serialized1.as_bytes())
                            {
                                println!("Publish error: {:?}", e);
                            };
                            if let Err(e) = swarm
                                .behaviour_mut()
                                .publish(topic_delegate_task.clone(), serialized2.as_bytes())
                            {
                                println!("Publish error: {:?}", e);
                            };

                        },


                        "discovery" => {

                            println!("discovery message from {:?}", peer_id);
                        }

                        _ => println!("Unknown topic"),
                    };
                },

                SwarmEvent::NewListenAddr { address, .. } => {
                    println!("Listening on {:?}", address);
                }
                _ => {}
            }
        }
    }
}
