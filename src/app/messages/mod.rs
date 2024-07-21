//! Main messages.
//!
use std::net::{SocketAddr, TcpStream};

use crate::redis::{sentinel::MasterChangeNotification, stream::network::NetworkStream};

/// Message to communicate with main loop
pub struct MainLoopEvent {
    /// If new client is here
    pub new_client: Option<(TcpStream, SocketAddr)>,
    /// If master change
    pub master_change: Option<MasterChangeNotification>,
    /// Worker get and release client
    pub worker_message: Option<GetAndReleaseClient>,
}

impl MainLoopEvent {
    /// Create message to notify new client is coming
    pub fn new_client(tcp_stream: TcpStream, socket_addr: SocketAddr) -> Self {
        Self {
            new_client: Some((tcp_stream, socket_addr)),
            master_change: None,
            worker_message: None,
        }
    }

    /// Create message to notify that the master address change
    pub fn master_change(new_master: MasterChangeNotification) -> Self {
        Self {
            new_client: None,
            master_change: Some(new_master),
            worker_message: None,
        }
    }

    pub fn worker_get_client(name: String, client: Option<ClientConnectionParameter>) -> Self  {
        Self {
            new_client: None,
            master_change: None,
            worker_message: Some(GetAndReleaseClient {
                worker_id: name,
                client_to_release: client
            }),
        }
    }
}

/// Get and release client
pub struct GetAndReleaseClient {
    /// The worker id
    pub worker_id: String,
    /// Client to release
    pub client_to_release: Option<ClientConnectionParameter>,
}

/// Client connection
pub struct ClientConnectionParameter {
    /// Unique id of client
    pub id: String,
    /// Client socket
    pub client_addr: SocketAddr,
    /// Client stream
    pub client_stream: NetworkStream,
    /// Redis stream
    pub redis_stream: NetworkStream
}