use std::time::Duration;
use futures::task::Poll;
use futures::task::Context;
use std::thread;
use futures::task::Waker;
use async_std::stream::Stream;
use core::pin::Pin;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ndarray::{Array2, Axis, concatenate};
use libp2p::PeerId;
use serde::{Serialize, Deserialize};


#[derive(Debug)]
pub struct MothershipState {
    pub position: Coordinate,
    pub mission_status: MissionStatus,
    pub mission_area: Option<Array2<u32>>,
    pub tasks: Arc<Mutex<VecDeque<Coordinate>>>,
    pub delegate_tasks: DelegateTasks,
}

#[derive(Debug)]
pub struct MinionState  {
    pub heartbeat: bool,
    pub ready: bool,
    pub position: Coordinate,
    pub poi: bool,
    pub mission_area: Option<Array2<u32>>,
    pub waker: Option<Waker>,
}

#[derive(Debug)]
pub struct MinionHeartbeat {
    pub position: Coordinate,
    pub poi: bool,
}

#[derive(Debug)]
pub struct MinionStream {
    shared_state: Arc<Mutex<MinionState>>,
}

#[derive(Debug)]
pub enum MissionStatus {
    Pending,
    InProgress,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Coordinate {
    pub fn inc_x(&mut self) {
        self.x = self.x + 1;
    }
}

// Struct used by mothership to keep track of minions
#[derive(Debug, Serialize, Deserialize)]
pub struct Minion {
    pub peer_id: PeerId,
    pub position: Coordinate,
}

#[derive(Debug)]
pub struct DelegateTasks {
    pub minions: HashMap<PeerId, Coordinate>,
    pub total: u32, // This is set once the mission is received, based on the number of subscribed minions.
    pub complete: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DelegateTaskMessage {
    pub peer_id: PeerId,
    pub global_coordinates: Coordinate,
    pub area: Array2<u32>,
}

impl Stream for MinionStream {

    type Item = MinionHeartbeat;

    fn poll_next(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {

        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.ready { // This should be moved to the new method, so that movement is first commenced when ready, currently polling is commenced when ready.

            if shared_state.heartbeat {
                shared_state.heartbeat = false;
                return Poll::Ready(Some(MinionHeartbeat {
                    position: shared_state.position.clone(),
                    poi: shared_state.poi.clone(),
                }));
            } else {
                shared_state.waker = Some(cx.waker().clone());
                return Poll::Pending;
            }
        } else {
            shared_state.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
    }
}

impl MinionStream {

    pub fn new(shared_state: Arc<Mutex<MinionState>>) -> Self {

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                let mut shared_state = thread_shared_state.lock().unwrap();

                // Advance to next position
                shared_state.position.inc_x();
                if shared_state.position.x % 2 == 0 {
                    shared_state.poi = true;
                } else {
                    shared_state.poi = false;
                };
                // Tell comms to poll again.
                shared_state.heartbeat = true;
                if let Some(waker) = shared_state.waker.take() {
                    waker.wake()
                }
            }
        });

        MinionStream { shared_state }
    }
}

pub fn mothership_bot (tasks: Arc<Mutex<VecDeque<Coordinate>>>) {
    loop {
        let mut tasks = tasks.lock().unwrap();
        if let Some(task) = tasks.pop_front() {
            drop(tasks);
            println!("Running pick up on {:?}", task);
            // Do pickup with robot
        } else {
            drop(tasks);
            println!("No more tasks");
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

pub fn split_mission_area(area: Array2<u32>, minion_count: usize) -> Vec<([i32; 2], Array2<u32>)> {
    let (axis, axis_size) = area.shape().iter().enumerate().max_by_key(|(_,v)| *v).unwrap();
    println!("Largest axis: {:?} with size: {:?}", axis, axis_size);

    if minion_count > 1 {
        let splits = axis_size / minion_count;
        let rem = axis_size % minion_count; 

        if rem > 0 {
            let mut split = area.axis_chunks_iter(Axis(axis), splits);
            let last1 = split.next_back().unwrap(); // `n-1`th element
            let last2 = split.next_back().unwrap(); // `n-2`th element

            let split = split.map(|x| x.to_owned());
            let joint = concatenate(Axis(axis), &[last2, last1]).unwrap();

            let areas = split.chain([joint]).collect::<Vec<_>>();

            let x = areas.iter().clone();
            let x = x.map(|value| value.to_owned());
            let mut step = splits as i32;
            let mut origins = vec![[0i32,0]; x.len()];
            for i in origins.iter_mut().skip(1) {
                i[axis] += step;
                step += step;
            };
            let y = origins.into_iter().zip(x);
            
            return y.collect::<Vec<_>>();

        } else {
            let areas = area.axis_chunks_iter(Axis(axis), splits);
            let x = areas.map(|value| value.to_owned());
            let mut step = splits as i32;
            let mut origins = vec![[0i32,0]; x.len()];
            for i in origins.iter_mut().skip(1) {
                i[axis] += step;
                step += step;
            };
            let y = origins.into_iter().zip(x);
            return y.collect::<Vec<_>>();
        }

    } else {
        return vec![([0,0], area)];
    }

}