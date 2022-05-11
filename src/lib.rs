use async_std::stream::Stream;
use core::pin::Pin;
use futures::task::Context;
use futures::task::Poll;
use futures::task::Waker;
use libp2p::PeerId;
use ndarray::{concatenate, Array2, Axis};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::ops::Add;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::vec::IntoIter;

#[derive(Debug)]
pub struct MothershipState {
    pub position: Coordinate,
    pub mission_status: MissionStatus,
    pub mission_area: Option<Array2<u32>>,
    pub delegate_tasks: DelegateTasks,
    pub points_of_interest: VecDeque<Coordinate>,
}

#[derive(Debug)]
pub struct MinionState {
    pub heartbeat: bool,
    pub ready: bool,
    pub global_position: Coordinate,
    pub local_position: Coordinate,
    pub area_exhausted: bool,
    pub poi: bool,
    pub mission_area: Option<IntoIter<((i32, i32), u32)>>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
    pub global_coordinates: Coordinate,
    pub area: Array2<u32>,
}

impl Add for Coordinate {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Stream for MinionStream {
    type Item = MinionHeartbeat;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut shared_state = self.shared_state.lock().unwrap();

        if shared_state.heartbeat {
            if shared_state.area_exhausted {
                shared_state.heartbeat = false;
                return Poll::Ready(None);
            }
            shared_state.heartbeat = false;
            return Poll::Ready(Some(MinionHeartbeat {
                position: shared_state.local_position.clone(),
                poi: shared_state.poi.clone(),
            }));
        } else {
            shared_state.waker = Some(cx.waker().clone());
            return Poll::Pending;
        }
    }
}

impl MinionStream {
    pub fn new(shared_state: Arc<Mutex<MinionState>>) -> Self {
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(100));
            let mut shared_state = thread_shared_state.lock().unwrap();

            if shared_state.ready {
                match &mut shared_state.mission_area {
                    Some(area) => {
                        let current_location = area.next();
                        match current_location {
                            Some(((x, y), poi)) => {
                                shared_state.local_position = Coordinate { x, y };
                                shared_state.poi = if poi != 0 { true } else { false };
                                shared_state.heartbeat = true;
                                if let Some(waker) = shared_state.waker.take() {
                                    waker.wake()
                                };
                            }
                            None => {
                                shared_state.heartbeat = true;
                                shared_state.area_exhausted = true;
                                break;
                            }
                        }
                    }
                    None => {
                        panic!("No mission area!");
                    }
                }
            }
        });

        MinionStream { shared_state }
    }
}

pub fn split_mission_area(area: Array2<u32>, minion_count: usize) -> Vec<([i32; 2], Array2<u32>)> {
    let (axis, axis_size) = area
        .shape()
        .iter()
        .enumerate()
        .max_by_key(|(_, v)| *v)
        .unwrap();
    if minion_count > 1 {
        let splits = axis_size / minion_count;
        let rem = axis_size % minion_count;

        dbg!(splits, rem);

        if rem > 0 && rem < splits {
            let mut split = area.axis_chunks_iter(Axis(axis), splits);
            let last1 = split.next_back().unwrap(); // `n-1`th element
            let last2 = split.next_back().unwrap(); // `n-2`th element

            let split = split.map(|x| x.to_owned());
            let joint = concatenate(Axis(axis), &[last2, last1]).unwrap();

            let areas = split.chain([joint]).collect::<Vec<_>>();

            let x = areas.iter().clone();
            let x = x.map(|value| value.to_owned());
            dbg!(x.len());
            let mut step = splits as i32;
            let mut origins = vec![[0i32, 0]; x.len()];
            for i in origins.iter_mut().skip(1) {
                i[axis] += step;
                step += splits as i32;
            }
            let y = origins.into_iter().zip(x);

            return y.collect::<Vec<_>>();

        } else if rem > 0 && rem > splits {
            let mut split = area.axis_chunks_iter(Axis(axis), splits);
            let last1 = split.next_back().unwrap(); // `n-1`th element
            let last2 = split.next_back().unwrap(); // `n-2`th element
            let last3 = split.next_back().unwrap(); // `n-3`th element

            let split = split.map(|x| x.to_owned());
            let joint = concatenate(Axis(axis), &[last3, last2, last1]).unwrap();

            let areas = split.chain([joint]).collect::<Vec<_>>();

            let x = areas.iter().clone();
            let x = x.map(|value| value.to_owned());
            dbg!(x.len());
            let mut step = splits as i32;
            let mut origins = vec![[0i32, 0]; x.len()];
            for i in origins.iter_mut().skip(1) {
                i[axis] += step;
                step += splits as i32;
            }
            let y = origins.into_iter().zip(x);

            return y.collect::<Vec<_>>();

        } else {

            let areas = area.axis_chunks_iter(Axis(axis), splits);
            let x = areas.map(|value| value.to_owned());
            let mut step = splits as i32;
            let mut origins = vec![[0i32, 0]; x.len()];
            for i in origins.iter_mut().skip(1) {
                i[axis] += step;
                step += splits as i32;
            }
            let y = origins.into_iter().zip(x);
            return y.collect::<Vec<_>>();
        }
    } else {
        return vec![([0, 0], area)];
    }
}
