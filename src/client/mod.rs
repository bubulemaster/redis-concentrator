//! This module contains routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::types::RedisError;
use std::net::TcpListener;
use std::thread;

pub fn watch_client(config: &Config, logger_client: slog::Logger) -> Result<(), RedisError> {
    // TODO set non blocking mode to incomming connection
    // then add this connection to redis_box
    let listener = match TcpListener::bind(&config.bind) {
        Ok(l) => l,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    thread::spawn(move || loop {
        let (client_stream, client_addr) = match listener.accept() {
            Ok(d) => d,
            Err(e) => {
                error!(
                    logger_client,
                    "Error when establish client connection {:?}.", e
                );

                continue;
            }
        };

        debug!(
            logger_client,
            "New client from {}:{}",
            client_addr.ip().to_string(),
            client_addr.port()
        );
    });

    Ok(())
}
