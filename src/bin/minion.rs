use std::error::Error;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use futures::{prelude::*, select};
use libp2p::{gossipsub, identity, swarm::SwarmEvent, Multiaddr, PeerId};
use libp2p::gossipsub::{GossipsubEvent, IdentTopic as Topic, MessageAuthenticity, ValidationMode};

use libd2d::{ DelegateTaskMessage, MinionState, Coordinate, MinionStream };


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // Set initial state
    let state = Arc::new(Mutex::new(MinionState {
        heartbeat: false,
        ready: false,
        global_position: Coordinate { x: -5, y: -5 },
        local_position: Coordinate { x: 0, y: 0 },
        area_exhausted: false,
        poi: false,
        mission_area: None,
        waker: None
    }));

    // Create a random PeerId
    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    // Set up an encrypted TCP Transport over the Mplex and Yamux protocols
    let transport = libp2p::development_transport(local_key.clone()).await?;

    let topic_delegate_task = Topic::new("delegate_task");
    let topic_poi = Topic::new("poi");
    let topic_task_complete = Topic::new("task_complete");

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
        gossipsub.subscribe(&topic_poi).unwrap();
        gossipsub.subscribe(&topic_task_complete).unwrap();

        libp2p::Swarm::new(transport, gossipsub, local_peer_id)
    };

    let address: Multiaddr = "/ip4/127.0.0.1/tcp/60740".parse().unwrap();
    match swarm.dial(address.clone()) {
        Ok(_) => println!("Dialed {:?}", address),
        Err(e) => println!("Dial {:?} failed: {:?}", address, e),
    };

    let thread_shared_state = Arc::clone(&state);
    let poi_stream = MinionStream::new(thread_shared_state);
    let mut poi_stream = poi_stream.chain(futures::stream::iter(std::iter::from_fn(|| { dbg!("ended"); None }))).fuse();

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
                                    let mut state = state.lock().unwrap();

                                    let indexed = task.area.indexed_iter();
                                    let collected = indexed.collect::<Vec<_>>();
                                    let owned_iter = collected.into_iter();
                                    let x = owned_iter.map(|((i, j), k)| ((i as i32, j as i32), *k));
                                    state.mission_area = Some(x.collect::<Vec<_>>().into_iter());

                                    state.global_position = task.global_coordinates;

                                    state.ready = true;
                                }
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

        x = poi_stream.next() => {
            match x {
                Some(x) => {
                     if x.poi {
                        let state = state.lock().unwrap();
                        let adjusted_poi = x.position + state.global_position;
                        drop(state);
                        let poi_serialized = serde_json::to_string(&adjusted_poi).unwrap();
                        if let Err(e) = swarm
                            .behaviour_mut()
                            .publish(topic_poi.clone(), poi_serialized.as_bytes())
                        {
                            println!("Publish error: {:?}", e);
                        }
                    };
                    let state = state.lock().unwrap();
                    if state.area_exhausted {
                        println!("task-complete");
                        drop(state);
                        if let Err(e) = swarm
                            .behaviour_mut()
                            .publish(topic_poi.clone(), "Done".as_bytes())
                        {
                            println!("Publish error: {:?}", e);
                        }
                    }
                },
                None => {
                    dbg!("NONE");
                }
            }
           

        }

        }
    }
}