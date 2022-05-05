use std::collections::HashMap;
use std::collections::VecDeque;
use std::error::Error;
use std::time::Duration;
use futures::{prelude::*, select};
use libp2p::{gossipsub, identity, swarm::SwarmEvent, PeerId};
use libp2p::gossipsub::{GossipsubEvent, IdentTopic as Topic, MessageAuthenticity, ValidationMode};
use serde_json;
use ndarray::Array2;

use libd2d::{MothershipState, MissionStatus, Coordinate, DelegateTasks, DelegateTaskMessage, split_mission_area};


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    // Set initial state
    let mut state = MothershipState {
        position: Coordinate { x: -1, y: -1 },
        mission_status: MissionStatus::Pending,
        mission_area: None,
        delegate_tasks: DelegateTasks {
            minions: HashMap::new(),
            total: 0,
            complete: 0,
        },
        points_of_interest: VecDeque::new(),
    };

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    // Create a Gossipsub topic
    let topic_new_mission = Topic::new("new_mission");
    let topic_delegate_task = Topic::new("delegate_task");
    let topic_poi = Topic::new("poi");
    let topic_task_complete = Topic::new("task_complete");

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

        gossipsub.subscribe(&topic_new_mission).unwrap();
        gossipsub.subscribe(&topic_delegate_task).unwrap();
        gossipsub.subscribe(&topic_poi).unwrap();
        gossipsub.subscribe(&topic_task_complete).unwrap();

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

                        "new_mission" => {

                            let area: Array2<u32> = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();

                            // Update state
                            state.mission_status = MissionStatus::InProgress;
                            state.mission_area = Some(area.clone());
                            let minion_count = state.delegate_tasks.minions.len();
                            state.delegate_tasks.total = minion_count as u32;

                            // Split up area amongst minions
                            let splits = split_mission_area(area.clone(), minion_count);
                            let zipped = splits.iter().zip(state.delegate_tasks.minions.clone());

                            println!("{:?}", splits);

                            for (subarea, minion) in zipped {
                                let task_message = DelegateTaskMessage {
                                    peer_id: minion.0.clone(),
                                    global_coordinates: Coordinate { x: subarea.0[0], y: subarea.0[1] },
                                    area: subarea.1.to_owned(),
                                };
                                let task_message = serde_json::to_string(&task_message).unwrap();
                                if let Err(e) = swarm
                                    .behaviour_mut()
                                    .publish(topic_delegate_task.clone(), task_message.as_bytes())
                                {
                                    println!("Publish error: {:?}", e);
                                }
                            }
                        },

                        "poi" => {
                            let poi: Coordinate = serde_json::from_str(&String::from_utf8_lossy(&message.data)).unwrap();
                            state.points_of_interest.push_front(poi);
                        },

                        "task_complete" => {
                            println!("task-complete");
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
