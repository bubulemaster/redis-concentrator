//! Main application loop.
//! Wait message from watch_new_client_connection and workers and dispatch client to worker.
//!
use std::{collections::HashMap, net::{SocketAddr, TcpStream}, sync::mpsc::Receiver};

use crate::redis::{sentinel::MasterChangeNotification, stream::network::NetworkStream};

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

pub fn run_main_loop(rx_main_loop_message: Receiver<MainLoopEvent>, redis_addr: String, logger: slog::Logger)  {
    debug!(logger, "Start main event loop");

    let mut client_map:HashMap<String, (NetworkStream, NetworkStream)> = HashMap::new();
    let mut redis_master_addr = String::from(redis_addr);
}