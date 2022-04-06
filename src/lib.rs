use std::collections::VecDeque;
use std::sync::mpsc;


// Define datatypes

pub struct MothershipState {
    pub mission: Mission,
    pub tasks: VecDeque<Point>,
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

pub fn search (tx: mpsc::Sender<()>, rx: mpsc::Receiver<Point>) {

    loop {
        if let Ok(point) = rx.try_recv() {
            println!("Got: {:?}", point);
            std::thread::sleep(std::time::Duration::from_secs(3));
            println!("searched: {:?}", point);
        }
        tx.send(()).unwrap();
    }


}