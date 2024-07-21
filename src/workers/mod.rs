//! This module contains routine of worker that read data from client to write to redis,
//! and read data from redis to write to client
use std::collections::HashMap;
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};
use log::{debug, error};
use uuid::Uuid;
use crate::app::messages::{MainLoopEvent, ClientConnectionParameter};

/// Struct to send message to worker
pub struct WorkerEvent {
    // If read/write to/form client/redis
    pub client: Option<ClientConnectionParameter>,
    // If worker must be down
    pub shutdown: bool
}

/// To send message to worker
pub type WorkerEventReceiver = Sender<WorkerEvent>;

pub fn create_one_worker(name: String, tx_main_loop_message: Sender<MainLoopEvent>) -> WorkerEventReceiver {
    // Channel to main loop
    let (tx_worker_message, rx_worker_message): (
        Sender<WorkerEvent>,
        Receiver<WorkerEvent>,
    ) = mpsc::channel();

    debug!("Start worker: {}", name);

    let _ = thread::Builder::new().name(name.clone()).spawn(move || {
        // Ask to main loop to get a new client
        tx_main_loop_message.send(MainLoopEvent::worker_get_client(name.clone(), None)).unwrap();

        match rx_worker_message.recv() {
            Ok(event) => {
                error!("Worker '{}' receive a client to read", name.clone());
            },
            Err(_) => {                
                error!("Worker '{}' can't get message from main loop cause his channel is closed", name.clone());
                // TODO send notification to main loop to close worker
            }
        };
    });

    tx_worker_message
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