//! This module contains routine of worker that read data from client to write to redis,
//! and read data from redis to write to client
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};
use log::debug;
use uuid::Uuid;
use crate::app::{MainLoopEvent, ClientConnectionParameter};

/// Struct to send message to worker
pub struct WorkerEvent {
    // If read/write to/form client/redis
    pub client: Option<ClientConnectionParameter>,
    // If worker must be down
    pub shutdown: bool
}

/// To send message to worker
pub type WorkerEventReceiver = Receiver<WorkerEvent>;

pub fn create_one_worker(name: String, tx_main_loop_message: Sender<MainLoopEvent>) -> WorkerEventReceiver {
    // Channel to main loop
    let (tx_main_loop_message, rx_main_loop_message): (
        Sender<WorkerEvent>,
        Receiver<WorkerEvent>,
    ) = mpsc::channel();

    debug!("Start worker: {}", name);

    // TODO
    // let _ = thread::Builder::new().name(name).spawn(move || {
    // });

    rx_main_loop_message
}

pub fn create_workers_pool(count: u8, tx_main_loop_message: &Sender<MainLoopEvent>) -> HashMap<String, WorkerEventReceiver> {
    let mut workers_map: HashMap<String, WorkerEventReceiver> = HashMap::new();

    for _ in 0..count { 
        let id = Uuid::new_v4();
        let name = format!("worker-{}", id);
        let name2 = name.clone();

        workers_map.insert(
            name,
            create_one_worker(name2, tx_main_loop_message.clone()));
    }

    workers_map
}