use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use ndarray::Array2;
use libp2p::PeerId;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct MothershipState {
    pub mission_status: MissionStatus,
    pub mission_area: Option<Array2<u32>>,
    pub position: Coordinate,
    pub tasks: Arc<Mutex<VecDeque<Coordinate>>>,
    pub delegate_tasks: DelegateTasks,
}

#[derive(Debug)]
pub enum MissionStatus {
    Pending,
    InProgress,
    Complete,
}

#[derive(Debug)]
pub struct Coordinate {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug)]
pub struct DelegateTasks {
    pub minions: Vec<PeerId>,
    pub total: u32, // This is set once the mission is received, based on the number of subscribed minions.
    pub complete: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DelegateTaskMessage {
    pub peer_id: PeerId,
    pub area: Array2<u32>,
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
