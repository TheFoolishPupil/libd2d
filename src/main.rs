use std::collections::VecDeque;
use std::thread;
use std::sync::mpsc;

use libd2d::{Mission, MothershipState, MissionStatus, Area, Point, DelegateTasks, search};


fn main() {

    // Set initial state
    let mut state = MothershipState {
        mission: Mission {
            status: MissionStatus::InProgress,
            area: Area {x1: 0, y1: 0, x2: 10, y2: 10},
        },
        tasks: VecDeque::from([Point {x:1, y:1} , Point {x:2, y:2} , Point {x:3, y:3}]),
        delegate_tasks: DelegateTasks {
            total: 0,
            complete: 0,
        }

    };

    // create robo thread
    let (robot_tx, robot_in_rx) = mpsc::channel();
    let (robot_out_tx, robot_rx) = mpsc::channel();
    thread::spawn(move || search(robot_tx, robot_rx));

    // // Send initial task
    // if let Some(x) = state.tasks.pop_front() {
    //     println!("Sending: {:?}", x);
    //     robot_out_tx.send(x).unwrap();
    // } else {
    //     // No more tasks in queue
    // }

    loop {

        if let Ok(_) = robot_in_rx.try_recv() { // Robo thread is done with current task
            if let Some(x) = state.tasks.pop_front() {
                println!("Sending: {:?}", x);
                robot_out_tx.send(x).unwrap();
            } else {
                // No more tasks in queue
            }
        } else {
            // println!("Robot busy");
        }


        // robot_out_tx.send(task).unwrap();

    }
}
