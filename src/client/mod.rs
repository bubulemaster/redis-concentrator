//! This module contains routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::types::RedisError;
use crate::node::create_stream_connection;
use crate::sentinel::MasterChangeNotification;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{Receiver, Sender};
use std::{thread, time};

/// Wait new client connection.
/// Create a new thread for do this.
pub fn watch_client(
    config: &Config,
    logger: slog::Logger,
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
                    logger,
                    "New client from {}:{}",
                    client_addr.ip().to_string(),
                    client_addr.port()
                );

                // Set non blocking mode to incoming connection
                if let Err(e) = client_stream.set_nonblocking(true) {
                    error!(
                        logger,
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
                error!(logger, "Error when establish client connection {:?}.", e);

                continue;
            }
        };
    });

    Ok(())
}

/// Create loop to copy data.
pub fn copy_data_from_client_to_redis(
    redis_master_addr: &str,
    logger: slog::Logger,
    rx_master_change: Receiver<MasterChangeNotification>,
    rx_new_client: Receiver<(TcpStream, SocketAddr)>,
) -> Result<(), RedisError> {
    let mut client_map = HashMap::new();
    let mut redis_master_addr = String::from(redis_master_addr);

    let sleep_duration = time::Duration::from_millis(200);

    loop {
        let msg_master_change = rx_master_change.try_recv();

        // TODO check error to see if thread is dead.
        if let Ok(msg) = msg_master_change {
            debug!(logger, "Master change: {:?}", msg);

            redis_master_addr = msg.new.clone();

            // TODO close previous client connection
        }

        manage_new_client_message(
            &logger,
            &rx_new_client,
            &mut client_map,
            &mut redis_master_addr,
        );

        manage_client_data(&logger, &mut client_map);

        thread::sleep(sleep_duration);
    }

    Ok(())
}

/// Manage data (copy) from/to client.
fn manage_client_data(
    logger: &slog::Logger,
    client_map: &mut HashMap<String, (TcpStream, TcpStream), RandomState>,
) {
    let mut buffer = [0u8; 2048];

    let mut remove_connection = Vec::new();
    // TODO to know if connection is closed, must send data then read
    for (key, stream) in client_map.iter_mut() {
        let (client_stream, client_redis_stream) = stream;

        // Copy data from client to redis master
        if let Ok(read_size) = client_stream.read(&mut buffer) {
            if let Ok(write_size) = client_redis_stream.write(&buffer[..read_size]) {
                if read_size != write_size {
                    debug!(logger, "Redis master have close connection for {}.", &key);

                    // In blocking mode, that mean socket is closed
                    remove_connection.push(key.clone());
                }
            }
        }

        // Copy data from client to redis master
        if let Ok(read_size) = client_redis_stream.read(&mut buffer) {
            if let Ok(write_size) = client_stream.write(&buffer[..read_size]) {
                if read_size != write_size {
                    debug!(logger, "Redis master stream is close for {}.", &key);

                    // In blocking mode, that mean socket is closed
                    remove_connection.push(key.clone());
                }
            }
        }
    }

    for key in remove_connection {
        let (client_stream, client_redis_stream) = client_map.remove(&key).unwrap();
        if let Ok(_) = client_stream.shutdown(Shutdown::Both) {
            // Other when we don't care
        }
    }
}

/// Manage client connection.
fn manage_new_client_message(
    logger: &slog::Logger,
    rx_new_client: &Receiver<(TcpStream, SocketAddr)>,
    client_map: &mut HashMap<String, (TcpStream, TcpStream), RandomState>,
    redis_master_addr: &mut String,
) {
    let msg_new_client = rx_new_client.try_recv();

    // TODO check error to see if thread is dead.
    if let Ok(msg) = msg_new_client {
        debug!(logger, "New client: {:?}", msg);

        // Create one connection to master per client
        if let Ok(client_redis_stream) = create_stream_connection(&redis_master_addr) {
            let (client_stream, client_addr) = msg;

            let key = format!("{}:{}", client_addr.ip().to_string(), client_addr.port());

            client_map.insert(key, (client_stream, client_redis_stream));
        } else {
            // TODO stop thread
            error!(logger, "Can't create new Redis master connection");
        }
    }
}
