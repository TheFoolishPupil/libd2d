use futures::task::Poll;
use futures::task::Context;
use futures::task::Waker;
use core::pin::Pin;
use std::collections::VecDeque;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use ndarray::Array2;
use libp2p::PeerId;
use serde::{Serialize, Deserialize};


#[derive(Debug)]
pub struct MothershipState {
    pub position: ActorPosition,
    pub mission_status: MissionStatus,
    pub mission_area: Option<Array2<u32>>,
    pub tasks: Arc<Mutex<VecDeque<Coordinate>>>,
    pub delegate_tasks: DelegateTasks,
    waker: Option<Waker>,
}

#[derive(Debug)]
pub struct MinionState  {
    pub position: ActorPosition,
    pub tasks: Arc<Mutex<VecDeque<Array2<u32>>>>,
    pub points_of_interest: Arc<Mutex<VecDeque<Coordinate>>>,
}

#[derive(Debug)]
pub struct ActorPosition {
    pub coordinates: Coordinate,
    pub orientation: f64,
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
    pub area: Array2<u32>,
}

impl futures::stream::Stream for MinionState {

    type Item = Coordinate;

    fn poll_next(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {

        let mut pois = self.points_of_interest.lock().unwrap();
        if let Some(poi) = pois.pop_front() {
            return Poll::Ready(Some(poi));
        } else {
            // self.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
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

pub fn minion_bot (tasks: Arc<Mutex<VecDeque<Array2<u32>>>>, points_of_interest:Arc<Mutex<VecDeque<Coordinate>>>) {
    loop {
        let mut tasks = tasks.lock().unwrap();
        if let Some(task) = tasks.pop_front() {
            drop(tasks);
            println!("Running search on {:?}", task);
            // Do search with robot
            {
                let mut pois = points_of_interest.lock().unwrap();
                pois.push_front(Coordinate {x:0, y:0});
            }
        } else {
            drop(tasks);
            println!("No more tasks");
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
