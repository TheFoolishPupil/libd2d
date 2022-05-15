// loop {
//     select! {
//         event = swarm.select_next_some() => match event {
//             SwarmEvent::Behaviour(GossipsubEvent::Message {
//                 propagation_source: _peer_id,
//                 message_id: _id,
//                 message,
//             }) => {
//                 match message.topic.as_str() {

//                     "some_topic" => {

//                         let mut pois = VecDeque::from(["a", "b", "c", "d"]);

//                         while !pois.is_empty() {
//                             let data = pois.pop_front();
//                             if let Err(e) = swarm
//                                 .behaviour_mut()
//                                 .publish(some_other_topic.clone(), data.as_bytes())
//                             {
//                                 println!("Publish error: {:?}", e);
//                             };
//                         }

//                     }
//                 }
//             }
//         }
//     }
// }


use std::collections::VecDeque;

fn main() {
    let mut pois = VecDeque::from(["a", "b", "c", "d"]);
    while !pois.is_empty() {
        let data = pois.pop_front();
        dbg!(data);
    }
}