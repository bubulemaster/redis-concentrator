//! Main application loop.
//! Wait message from watch_new_client_connection and workers and dispatch client to worker.
//!
use std::{collections::HashMap, net::{SocketAddr, TcpStream}, sync::mpsc::Receiver};
use crate::redis::{node::create_redis_stream_connection, sentinel::MasterChangeNotification, stream::network::NetworkStream};

/// Message to communicate with main loop
pub struct MainLoopEvent {
    // If new client is here
    pub new_client: Option<(TcpStream, SocketAddr)>,
    // If master change
    pub master_change: Option<MasterChangeNotification>,
}

impl MainLoopEvent {
    /// Create message to notify new client is coming
    pub fn new_client(tcp_stream: TcpStream, socket_addr: SocketAddr) -> Self {
        Self {
            new_client: Some((tcp_stream, socket_addr)),
            master_change: None,
        }
    }

    /// Create message to notify that the master address change
    pub fn master_change(new_master: MasterChangeNotification) -> Self {
        Self {
            new_client: None,
            master_change: Some(new_master),
        }
    }
}

/// Client connection
struct ClientConnectionParameter {
    /// Client socket
    pub client_addr: SocketAddr,
    /// Client stream
    pub client_stream: NetworkStream,
    /// Redis stream
    pub redis_stream: NetworkStream
}

pub fn run_main_loop(rx_main_loop_message: Receiver<MainLoopEvent>, redis_addr: String, logger: slog::Logger) -> Result<(), String> {
    debug!(logger, "Start main event loop");

    let mut client_map: HashMap<String, ClientConnectionParameter> = HashMap::new();
    let mut redis_master_addr = String::from(redis_addr);

    loop {
        match rx_main_loop_message.recv() {
            Ok(event) => manage_message(event, &mut client_map, &mut redis_master_addr, &logger),
            Err(_) => return Err(String::from("Main channel is closed!"))
        }
    }
}

fn manage_message(event: MainLoopEvent, client_map: &mut HashMap<String, ClientConnectionParameter>, redis_master_addr: &mut String, logger: &slog::Logger) {
    if let Some(client) = event.new_client {
        let (client_stream, client_addr) = client;
        manage_message_new_client(client_addr, client_stream, client_map, redis_master_addr, logger);
    }
}

fn manage_message_new_client(client_addr: SocketAddr, client_stream: TcpStream, client_map: &mut HashMap<String, ClientConnectionParameter>, redis_master_addr: &String, logger: &slog::Logger) {
    let key = format!("{}:{}", client_addr.ip().to_string(), client_addr.port());

    debug!(logger, "Main loop receive a new client from {}", key);

    // Create one connection to master per client
    if let Ok(client_redis_stream) = create_redis_stream_connection(redis_master_addr) {
        client_map.insert(
            key,
            ClientConnectionParameter {
                client_addr: client_addr,
                client_stream: NetworkStream::new(client_stream),
                redis_stream: client_redis_stream
            }
        );
    } else {
        error!(logger, "Can't create new Redis master connection");
    }
}