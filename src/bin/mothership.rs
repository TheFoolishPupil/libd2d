use std::collections::HashMap;
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

use libd2d::{MothershipState, Minion, MissionStatus, Coordinate, DelegateTasks, DelegateTaskMessage, mothership_bot, split_mission_area};


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    // Set initial state
    let mut state = MothershipState {
        position: Coordinate { x: -1, y: -1 },
        mission_status: MissionStatus::Pending,
        mission_area: None,
        tasks: Arc::new(Mutex::new(VecDeque::new())),
        delegate_tasks: DelegateTasks {
            minions: HashMap::new(),
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
    let topic_heartbeat = Topic::new("heartbeat");
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

        gossipsub.subscribe(&topic_heartbeat).unwrap();
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
                    state.delegate_tasks.minions.entry(peer_id).or_insert(Coordinate {x:0, y:0});
                    // println!("{:?}", state);
                },

                SwarmEvent::Behaviour(GossipsubEvent::Message {
                    propagation_source: _peer_id,
                    message_id: _id,
                    message,
                }) => {
                    match message.topic.as_str() {

                        "heartbeat" => {
                            let heartbeat: Minion = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            println!("{:?}", heartbeat);
                        },

                        "new_mission" => {

                            // Update state
                            state.mission_status = MissionStatus::InProgress;
                            
                            let area: Array2<u32> = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            state.mission_area = Some(area.clone());

                            let minion_count = state.delegate_tasks.minions.len();

                            state.delegate_tasks.total = minion_count as u32;

                            let splits = split_mission_area(area.clone(), minion_count);

                            // Zip area spits with connected minions
                            let zipped = splits.iter().zip(state.delegate_tasks.minions.clone());

                            for (subarea, minion) in zipped {
                                let task_message = DelegateTaskMessage {
                                    peer_id: minion.0.clone(),
                                    area: subarea.to_owned(),
                                };
                                let task_message = serde_json::to_string(&task_message).unwrap();
                                if let Err(e) = swarm
                                    .behaviour_mut()
                                    .publish(topic_delegate_task.clone(), task_message.as_bytes())
                                {
                                    println!("Publish error: {:?}", e);
                                }
                            }
                            // REFACTOR FROM
                            // TODO: Splitting only exactly for 2 minions. 
                            // This should be generalized for N minions. 
                            // Create a function that returns an iterator?
                            // let (dim_x, _) = area.dim();
                            // let subareas = area
                            //     .view()
                            //     .split_at(ndarray::Axis(0), dim_x/minion_count);

                            // let subareas = vec![subareas.0, subareas.1];

                            // let mut split_count = 0;
                            // for (peer_id, _) in state.delegate_tasks.minions.iter() {
                            //     let task_msg = DelegateTaskMessage {
                            //         peer_id: peer_id.clone(),
                            //         area: subareas[split_count].to_owned()
                            //     };
                            //     split_count += 1;
                            //     let task_msg = serde_json::to_string(&task_msg).unwrap();
                            //     if let Err(e) = swarm
                            //         .behaviour_mut()
                            //         .publish(topic_delegate_task.clone(), task_msg.as_bytes())
                            //     {
                            //         println!("Publish error: {:?}", e);
                            //     }
                            // }
                            // REFACTOR TO

                        },

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
