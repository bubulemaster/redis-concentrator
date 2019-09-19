//! This module contains routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::types::RedisError;
use crate::node::create_stream_connection;
use crate::sentinel::MasterChangeNotification;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};

/// Wait new client connection.
/// Create a new thread for do this.
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

                // Set non blocking mode to incoming connection
                if let Err(e) = client_stream.set_nonblocking(true) {
                    error!(
                        logger_client,
                        "Impossible to set client is non blocking mode from {}:{} cause {:?}",
                        client_addr.ip().to_string(),
                        client_addr.port(),
                        e
                    );

                    continue;
                }

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

/// Create loop to copy data.
pub fn copy_data_from_client_to_redis(
    redis_master_addr: &str,
    logger_redis_master: slog::Logger,
    rx_master_change: Receiver<MasterChangeNotification>,
    rx_new_client: Receiver<(TcpStream, SocketAddr)>,
) -> Result<(), RedisError> {
    let mut client_map = HashMap::new();
    let mut buffer = [0u8; 2048];
    let mut redis_stream = create_stream_connection(redis_master_addr)?;

    let sleep_duration = time::Duration::from_millis(200);

    loop {
        let msg_master_change = rx_master_change.try_recv();

        // TODO check error to see if thread is dead.
        if let Ok(msg) = msg_master_change {
            debug!(logger_redis_master, "Master change: {:?}", msg);

            redis_stream = create_stream_connection(&msg.new)?;

            // TODO close previous client connection
        }

        let msg_new_client = rx_new_client.try_recv();

        // TODO check error to see if thread is dead.
        if let Ok(msg) = msg_new_client {
            debug!(logger_redis_master, "New client: {:?}", msg);
            // TODO create one connection to master per client

            let (client_stream, client_addr) = msg;

            let key = format!("{}:{}", client_addr.ip().to_string(), client_addr.port());

            client_map.insert(key, client_stream);
        }

        for (key, stream) in client_map.iter_mut() {
            // Copy data from client to redis master
            if let Ok(size) = stream.read(&mut buffer) {
                if let Err(e) = redis_stream.write_all(&buffer[..size]) {
                    error!(
                        logger_redis_master,
                        "Write error to Redis master from {} with error: {:?}", &key, e
                    );
                }
            }

            // Copy data from client to redis master
            if let Ok(size) = redis_stream.read(&mut buffer) {
                if let Err(e) = stream.write_all(&buffer[..size]) {
                    warn!(
                        logger_redis_master,
                        "Write error to {} from Redis master with error: {:?}", &key, e
                    );
                }
            }
        }

        thread::sleep(sleep_duration);
    }

    Ok(())
}
