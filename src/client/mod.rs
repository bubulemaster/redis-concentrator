//! This module contains routine to watch clients.
//!
use crate::app::messages::MainLoopEvent;
use crate::config::Config;
use crate::redis::types::RedisError;
use std::net::TcpListener;
use std::sync::mpsc::Sender;
use std::thread;
use log::{error, info, debug, warn};

/// Wait new client connection.
/// Create a new thread for do this.
pub fn watch_new_client_connection(
    config: &Config,
    tx_new_client: Sender<MainLoopEvent>,
) -> Result<(), RedisError> {
    info!("Listen connection to {}", &config.bind);

    let listener = match TcpListener::bind(&config.bind) {
        Ok(l) => l,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    thread::spawn(move || loop {
        debug!("watch_new_client_connection(): Wait a new client");

        match listener.accept() {
            Ok(d) => {
                let (client_stream, client_addr) = d;

                debug!(
                    "New client from {}:{}",
                    client_addr.ip().to_string(),
                    client_addr.port()
                );

                // Set non blocking mode to incoming connection
                if let Err(e) = client_stream.set_nonblocking(true) {
                    error!(
                        "Impossible to set client is non blocking mode from {}:{} cause {:?}",
                        client_addr.ip().to_string(),
                        client_addr.port(),
                        e
                    );

                    continue;
                }

                if let Err(e) = client_stream.set_nodelay(true) {
                    warn!(
                        "Impossible to set client is no delay mode from {}:{} cause {:?}",
                        client_addr.ip().to_string(),
                        client_addr.port(),
                        e
                    );
                }

                tx_new_client.send(MainLoopEvent::new_client(client_stream, client_addr)).unwrap();
            }
            Err(e) => {
                error!("Error when establish client connection {:?}.", e);

                continue;
            }
        };
    });

    Ok(())
}

