//! Main application loop.
//! Wait message from watch_new_client_connection and workers and dispatch client to worker.
//!
use std::{collections::HashMap, net::{SocketAddr, TcpStream}, sync::mpsc::Receiver};
use log::{debug, error};
use messages::{GetAndReleaseClient, ClientConnectionParameter, MainLoopEvent};
use uuid::Uuid;


pub mod messages;


use crate::{redis::{node::create_redis_stream_connection, stream::network::NetworkStream}, workers::WorkerEventReceiver};



pub fn run_main_loop(rx_main_loop_message: Receiver<MainLoopEvent>, redis_addr: String, workers_map: HashMap<String, WorkerEventReceiver>) -> Result<(), String> {
    debug!("Start main event loop");

    let mut clients: Vec<ClientConnectionParameter> = vec![]; 
    let mut redis_master_addr = String::from(redis_addr);
    let mut workers: HashMap<String, WorkerEventReceiver> = HashMap::new();

    loop {
        match rx_main_loop_message.recv() {
            Ok(event) => manage_message(event, &mut redis_master_addr, &mut clients, &mut workers),
            Err(_) => return Err(String::from("Main channel is closed!"))
        }
    }
}

fn manage_message(event: MainLoopEvent, redis_master_addr: &mut String, clients: &mut Vec<ClientConnectionParameter>, workers_manager: &mut HashMap<String, WorkerEventReceiver>) {
    if let Some(client) = event.new_client {
        let (client_stream, client_addr) = client;
        
        if let Some(()) = manage_message_new_client(client_addr, client_stream, clients, redis_master_addr) {
            send_client_to_worker(clients, workers_manager);
        }
    } else if let Some(worker_message) = event.worker_message {
        manage_message_worker(worker_message, clients, workers_manager);
    }
}

fn manage_message_new_client(client_addr: SocketAddr, client_stream: TcpStream, clients: &mut Vec<ClientConnectionParameter>, redis_master_addr: &String) -> Option<()> {
    let key = format!("{}:{} - {}", client_addr.ip().to_string(), client_addr.port(), Uuid::new_v4());

    debug!("Main loop receive a new client from {}", key);

    // Create one connection to master per client
    if let Ok(client_redis_stream) = create_redis_stream_connection(redis_master_addr) {
        // Appends an element at the end of collection.
        clients.push(
            ClientConnectionParameter {
                id: key,
                client_addr: client_addr,
                client_stream: NetworkStream::new(client_stream),
                redis_stream: client_redis_stream
            }
        );

        Some(())
    } else {
        error!("Can't create new Redis master connection");

        None
    }
}

fn manage_message_worker(worker_message: GetAndReleaseClient, clients: &mut Vec<ClientConnectionParameter>, workers: &mut HashMap<String, WorkerEventReceiver>) {
    let worker_name = worker_message.worker_id;
    
    if clients.is_empty() {
        debug!("Worker '{}' want a client, but no client connected", worker_name.clone());
        return;
    }

    debug!("Worker '{}' want a client", worker_name.clone());

    // TODO
}

fn send_client_to_worker(clients: &mut Vec<ClientConnectionParameter>, workers: &mut HashMap<String, WorkerEventReceiver>) {
    // First check if we have client
    if clients.is_empty() {
        return;
    }

    // Second check if a worker is free
    if workers.is_empty() {
        return;
    }

    // TODO
}