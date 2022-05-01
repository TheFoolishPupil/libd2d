use std::error::Error;
use futures::{prelude::*, select};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use async_std::pin::Pin;
use async_std::task::{Context, Poll, Waker};
use async_std::stream;

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let shared_state = Arc::new(Mutex::new(SharedState {
        coordinate: None,
        counter: 0,
        waker: None,
    }));

    let stream_shared_state = shared_state.clone();
    let mut timer = TimerStream::new(stream_shared_state).fuse();

    loop {
        select! {
            x = timer.select_next_some() => {
                println!("{}", x);
            },
            // _ = heartbeat_interval.select_next_some() => {
            //     let shared_state_1 = shared_state.lock().unwrap();
            //     println!("HEARTBEATL {:?}", shared_state_1);
            // }
        }
    }
}

pub struct TimerStream {
    shared_state: Arc<Mutex<SharedState>>,
}

#[derive(Debug)]
pub struct SharedState {
    coordinate: Option<u32>,
    counter: u32,
    waker: Option<Waker>,
}

impl stream::Stream for TimerStream {
    type Item = u32;

    fn poll_next(
        self: Pin<&mut Self>, 
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>> {

        let mut shared_state = self.shared_state.lock().unwrap();

        match shared_state.coordinate {

            Some(coordinate) => {
                shared_state.coordinate = None;
                Poll::Ready(Some(coordinate))
            }
            None => {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
            }
        }
    }
}

impl TimerStream {

    // This should be a function that searches area, it should take an arc reference to the state, instead of owning it.
    pub fn new(shared_state: Arc<Mutex<SharedState>>) -> Self {

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1)); // Move to next space and perform checks for completion.
                let mut shared_state = thread_shared_state.lock().unwrap();
                // Update current location and if there are any darts.
                shared_state.coordinate = Some(shared_state.counter);
                shared_state.counter += 1;

                // Tell comms to poll again.
                if let Some(waker) = shared_state.waker.take() {
                    waker.wake()
                }
            }
        });

        TimerStream { shared_state }
    }
}