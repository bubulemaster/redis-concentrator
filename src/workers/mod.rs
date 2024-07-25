//! This module contains routine of worker that read data from client to write to redis,
//! and read data from redis to write to client
use std::thread;
use std::sync::mpsc::{self, Receiver, Sender};
use log::{debug, error};
use uuid::Uuid;
use crate::app::messages::{ClientConnectionParameter, MainLoopEvent};
use crate::redis::stream::network::NetworkStream;
use crate::redis::stream::RedisStream;

pub mod messages;

/// To send message to worker
pub type WorkerEventReceiver = Sender<messages::WorkerEvent>;

enum ErrorWorkerLoop {
    Stop,
    GetMessageFailed,
    ClientError(std::io::Error),
}

/// Create a worker
pub fn create_one_worker(name: String, tx_main_loop_message: Sender<MainLoopEvent>) {
    // Channel to main loop
    let (tx_worker_message, rx_worker_message): (
        WorkerEventReceiver,
        Receiver<messages::WorkerEvent>,
    ) = mpsc::channel();

    debug!("create_one_worker(): Start worker: {}", name);

    let _ = thread::Builder::new().name(name.clone()).spawn(move || {
        debug!("create_one_worker(): Ask to main loop to get a new client for first time");

        tx_main_loop_message.send(MainLoopEvent::worker_get_client(name.clone(), tx_worker_message.clone())).unwrap();

        loop {
            match run_worker_loop(&name, &rx_worker_message) {
                Ok(client) => 
                    // Release client and ask to main loop to get new client
                    tx_main_loop_message.send(MainLoopEvent::worker_send_and_get_client(name.clone(), client, tx_worker_message.clone())).unwrap(),
                Err(ErrorWorkerLoop::Stop) => return,
                Err(ErrorWorkerLoop::GetMessageFailed) => {
                    // TODO send to main loop to remove this worker

                    // Exit thread
                    return;
                }
                Err(ErrorWorkerLoop::ClientError(e)) => {
                    // TODO send message to remove client
                }
            }
        }
    });
}

/// Create a pool of worker
pub fn create_workers_pool(count: u8, tx_main_loop_message: &Sender<MainLoopEvent>) {
    for _ in 0..count { 
        let id = Uuid::new_v4();
        let name = format!("worker-{}", id);
        let name2 = name.clone();
        create_one_worker(name2, tx_main_loop_message.clone());
    }
}

#[inline]
fn run_worker_loop(name: &String, rx_worker_message: &Receiver<messages::WorkerEvent>) -> Result<ClientConnectionParameter, ErrorWorkerLoop> {
    debug!("run_worker_loop(): Worker wait message");

    match rx_worker_message.recv() {
        Ok(event) => {
            debug!("run_worker_loop(): Worker '{}' receive a client to read\nrun_worker_loop(): Event '{:?}'", name, event);

            if event.shutdown {
                return Err(ErrorWorkerLoop::Stop);
            }

            let mut client = event.client.unwrap();

            if let Err(e) = copy_data_from_client_to_redis(&mut client.client_stream, &mut client.redis_stream) {
                return Err(ErrorWorkerLoop::ClientError(e));
            }

            if let Err(e) = copy_data_from_redis_to_client(&mut client.client_stream, &mut client.redis_stream) {
                return Err(ErrorWorkerLoop::ClientError(e));
            }

            Ok(client)
        },
        Err(_) => {                
            error!("Worker '{}' can't get message from main loop cause his channel is closed", name);
            Err(ErrorWorkerLoop::GetMessageFailed)
        }
    }
}

#[inline]
fn copy_data_from_client_to_redis(client_stream: &mut NetworkStream, redis_stream: &mut NetworkStream) -> Result<(), std::io::Error> {
    // Copy data from client to redis master
    match client_stream.get_data(2048) {
        Ok(data) => {
            if let Err(e) = redis_stream.write(data.as_ref()) {
                return Err(e);
            }
        }
        Err(e) => return Err(e)
    };

    Ok(())
}

#[inline]
fn copy_data_from_redis_to_client(client_stream: &mut NetworkStream, redis_stream: &mut NetworkStream) -> Result<(), std::io::Error> {
    // Copy data from redis to client
    match redis_stream.get_data(2048) {
        Ok(data) => {
            if let Err(e) = client_stream.write(data.as_ref()) {
                return Err(e);
            }
        },
        Err(e) => return Err(e)
    }

    Ok(())
}