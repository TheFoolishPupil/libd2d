use std::error::Error;
use futures::{prelude::*, select};
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use async_std::pin::Pin;
use async_std::task::{Context, Poll, Waker};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut timer = TimerStream::new(Duration::from_secs(1)).fuse();

    loop {
        select! {
            x = timer.select_next_some() => {
                println!("{}", x);
            }
        }
    }
}

pub struct TimerStream {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    coordinate: Option<u32>,
    counter: u32,
    waker: Option<Waker>,
}

impl futures::stream::Stream for TimerStream {
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

    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            coordinate: None,
            counter: 0,
            waker: None,
        }));

        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            loop {
                thread::sleep(duration);
                let mut shared_state = thread_shared_state.lock().unwrap();
                shared_state.coordinate = Some(shared_state.counter);
                shared_state.counter += 1;
                if let Some(waker) = shared_state.waker.take() {
                    waker.wake()
                }
            }
        });

        TimerStream { shared_state }
    }
}