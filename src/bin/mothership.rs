use std::collections::VecDeque;
use std::error::Error;
use std::thread;
use std::sync::{Arc, Mutex};
use async_std::task;
use libd2d::{MothershipState, MissionStatus, Coordinate, DelegateTasks, mothership_bot};
use libd2d::comms::create_p2p_network;


#[async_std::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    // Set initial state
    let mut state = MothershipState {
        mission_status: MissionStatus::Pending,
        mission_area: None,
        position: Coordinate { x:0, y:0 },
        tasks: Arc::new(Mutex::new(VecDeque::new())),
        delegate_tasks: DelegateTasks {
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
    let robot_handle = thread::spawn(move || mothership_bot(tasks));

    // create libp2p thread
    let comm_handle = task::spawn(create_p2p_network());

    std::thread::sleep(std::time::Duration::from_secs(5));

    { // Aquire lock and create initial tasks
        let mut tasks = state.tasks.lock().unwrap();
        tasks.push_back(Coordinate {x:4, y:4});
        println!("{:?}", tasks);
    }
    state.delegate_tasks.total = 1;

    // Prevent main from exiting while thread is running
    robot_handle.join().unwrap();
    comm_handle.await?;

    Ok(())

}
