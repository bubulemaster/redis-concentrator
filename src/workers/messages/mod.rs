//! Worker messages.
//!
use crate::app::messages::ClientConnectionParameter;

/// Struct to send message to worker
#[derive(Debug)]
pub struct WorkerEvent {
    /// If read/write to/form client/redis
    pub client: Option<ClientConnectionParameter>,
    /// If worker must be down
    pub shutdown: bool
}

impl WorkerEvent {
    /// Create a message to send a client to a worker
    pub fn send_client(client: ClientConnectionParameter) -> Self {
        Self {
            client: Some(client),
            shutdown: false,
        }
    }
}