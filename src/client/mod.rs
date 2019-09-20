//! This module contains routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::stream::RedisStream;
use crate::lib::redis::types::RedisError;
use crate::node::create_redis_stream_connection;
use crate::sentinel::MasterChangeNotification;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener, TcpStream};
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

                if let Err(e) = client_stream.set_nodelay(true) {
                    warn!(
                        logger,
                        "Impossible to set client is no delay mode from {}:{} cause {:?}",
                        client_addr.ip().to_string(),
                        client_addr.port(),
                        e
                    );
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

    // TODO allow configuration
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
    client_map: &mut HashMap<String, (NetworkStream, NetworkStream)>,
) {
    let mut remove_connection = Vec::new();

    for (key, stream) in client_map.iter_mut() {
        let (client_stream, client_redis_stream) = stream;

        // Copy data from client to redis master
        match client_stream.get_data(2048) {
            Ok(data) => {
                if let Err(e) = client_redis_stream.write(data.as_ref()) {
                    if e.kind() != ErrorKind::BrokenPipe {
                        debug!(
                            logger,
                            "Unable to write data from client to redis server cause: {}", e
                        );
                    }

                    remove_connection.push(key.clone());
                }
            }
            Err(e) => {
                if e.kind() != ErrorKind::BrokenPipe {
                    debug!(logger, "Error when read data from client cause: {}", e);
                }

                remove_connection.push(key.clone());
            }
        }

        // Copy data from redis to client
        match client_redis_stream.get_data(2048) {
            Ok(data) => {
                if let Err(e) = client_stream.write(data.as_ref()) {
                    if e.kind() != ErrorKind::BrokenPipe {
                        debug!(
                            logger,
                            "Unable to write data from redis serveur to client cause: {}", e
                        );
                    }
                }
            }
            Err(e) => {
                if e.kind() != ErrorKind::BrokenPipe {
                    debug!(logger, "Error when read data from client cause: {}", e);
                }

                remove_connection.push(key.clone());
            }
        }
    }

    for key in remove_connection {
        debug!(logger, "Close connection {}", &key);
        client_map.remove(&key).unwrap();
    }
}

/// Manage client connection.
fn manage_new_client_message(
    logger: &slog::Logger,
    rx_new_client: &Receiver<(TcpStream, SocketAddr)>,
    client_map: &mut HashMap<String, (NetworkStream, NetworkStream)>,
    redis_master_addr: &mut String,
) {
    let msg_new_client = rx_new_client.try_recv();

    // TODO check error to see if thread is dead.
    if let Ok(msg) = msg_new_client {
        debug!(logger, "New client: {:?}", msg);

        // Create one connection to master per client
        if let Ok(client_redis_stream) = create_redis_stream_connection(&redis_master_addr) {
            let (client_stream, client_addr) = msg;

            let key = format!("{}:{}", client_addr.ip().to_string(), client_addr.port());

            client_map.insert(
                key,
                (NetworkStream::new(client_stream), client_redis_stream),
            );
        } else {
            // TODO stop thread
            error!(logger, "Can't create new Redis master connection");
        }
    }
}
