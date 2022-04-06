use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub mod comms;


pub struct MothershipState {
    pub mission: Mission,
    pub tasks: Arc<Mutex<VecDeque<Point>>>,
    pub delegate_tasks: DelegateTasks,
}

pub struct Mission {
    pub status: MissionStatus,
    pub area: Area,
}

pub enum MissionStatus {
    Pending,
    InProgress,
    Complete,
}

#[derive(Debug)]
pub struct Area {
    pub x1: u32,
    pub y1: u32,
    pub x2: u32,
    pub y2: u32,
}

#[derive(Debug)]
pub struct Point {
    pub x: u32,
    pub y: u32,
}

pub struct DelegateTasks {
    pub total: u32,
    pub complete: u32,
}

pub fn mothership_bot (tasks: Arc<Mutex<VecDeque<Point>>>) {
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
