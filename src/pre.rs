use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;


#[derive(Debug)]
struct Tasks {
    queue: VecDeque<u32>,
    active: Option<u32>,
}

fn main() {

    let tasks_arc = Arc::new(Mutex::new(Tasks {
        queue: VecDeque::from([1,2,3]),
        active: None,
    }));
    let mut handles = vec![];

    // loop {

    //     let mut tasks = tasks_arc.lock().unwrap();
    //     let tasks_arc = Arc::clone(&tasks_arc);
    //     let handle = thread::spawn(move || {
    //         let mut tasks = tasks_arc.lock().unwrap();
    //         tasks.active = tasks.queue.pop();
    //         println!("Result: {:?}", tasks);
    //     });
    //     handles.push(handle);
    // }

    for _ in 0..4 {
        let tasks_arc = Arc::clone(&tasks_arc);
        let handle = thread::spawn(move || {
            let mut tasks = tasks_arc.lock().unwrap(); // call this when I need to update in gossipsub loop
            tasks.active = tasks.queue.pop_front();
            tasks.queue.push_back(4);
            println!("Result: {:?}", tasks);
        });
        handles.push(handle);

    }


    for handle in handles {
        handle.join().unwrap();
    }

    // println!("Result: {:?}", *tasks_arc.lock().unwrap());


}
