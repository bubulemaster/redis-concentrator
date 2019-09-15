//! This module contains routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::types::RedisError;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::thread;

pub fn watch_client(
    config: &Config,
    logger_client: slog::Logger,
    tx_new_client: Sender<(TcpStream, SocketAddr)>,
) -> Result<(), RedisError> {
    let listener = match TcpListener::bind(&config.bind) {
        Ok(l) => l,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    thread::spawn(move || loop {
        match listener.accept() {
            Ok(d) => {
                let (client_stream, client_addr) = d;

                debug!(
                    logger_client,
                    "New client from {}:{}",
                    client_addr.ip().to_string(),
                    client_addr.port()
                );

                // TODO set non blocking mode to incomming connection
                tx_new_client.send((client_stream, client_addr)).unwrap();
            }
            Err(e) => {
                error!(
                    logger_client,
                    "Error when establish client connection {:?}.", e
                );

                continue;
            }
        };
    });

    Ok(())
}
