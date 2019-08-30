//! This module contain routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::types::{ErrorKind, RedisError, RedisValue};
use crate::lib::redis::{convert_to_string, RedisConnector};
use std::net::TcpStream;

/// Create a redis connector to sentinel.
fn create_redis_connection(address: &str) -> Result<NetworkStream, RedisError> {
    let tcp_stream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    // TODO allow to set value cause WouldBlock use it.
    let timeout = std::time::Duration::from_millis(200);

    if let Err(e) = tcp_stream.set_read_timeout(Some(timeout)) {
        return Err(RedisError::from_io_error(e));
    }

    if let Err(e) = tcp_stream.set_nonblocking(false) {
        return Err(RedisError::from_io_error(e));
    }

    Ok(NetworkStream::new(tcp_stream))
}

/// Create a raw tcp connection.
fn create_master_connection(address: &str) -> Result<TcpStream, RedisError> {
    let tcp_stream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    // TODO allow to set value cause WouldBlock use it.
    let timeout = std::time::Duration::from_millis(200);

    if let Err(e) = tcp_stream.set_read_timeout(Some(timeout)) {
        return Err(RedisError::from_io_error(e));
    }

    if let Err(e) = tcp_stream.set_nonblocking(false) {
        return Err(RedisError::from_io_error(e));
    }

    Ok(tcp_stream)
}

/// Check if no data available.
fn manage_redis_subscription_error(error: RedisError) -> Result<(), RedisError> {
    match error.kind() {
        ErrorKind::NoDataAvailable => return Ok(()),
        e => return Err(error),
    }
}

/// If we receive message.
fn manage_subscription_data(data: RedisValue, logger: &slog::Logger) -> Result<(), RedisError> {
    match data {
        RedisValue::Array(data) => {
            let msg_type = convert_to_string(data.get(0).unwrap())?;
            let channel = convert_to_string(data.get(1).unwrap())?;
            let data = data.get(2).unwrap();

            debug!(
                logger,
                "Receive message type: '{}' from channel: '{}' with data: '{:?}'",
                msg_type,
                channel,
                data
            );

            manage_subscription_message(&msg_type, &channel, &data, logger)
        }
        _ => Err(RedisError::from_message(
            "Impossible, subscription don't return array!",
        )),
    }
}

/// When receive a message from subscription.
fn manage_subscription_message(
    msg_type: &str,
    channel: &str,
    data: &RedisValue,
    logger: &slog::Logger,
) -> Result<(), RedisError> {
    match msg_type {
        "subscribe" => manage_subscription_message_type_subscribe(channel, data, logger),
        "message" => manage_subscription_message_type_message(channel, data, logger),
        e => {
            warn!(logger, "Unknow message type '{}'!", e);
            Ok(())
        }
    }
}

/// When receive a message type message from subscription.
fn manage_subscription_message_type_message(
    channel: &str,
    data: &RedisValue,
    logger: &slog::Logger,
) -> Result<(), RedisError> {
    /*
    1) "message"
    2) "+switch-master"
    3) "cluster_1 127.0.0.1 6001 127.0.0.1 6000" Groupe name : Old master -> New master
    */
    info!(logger, "{:?}", convert_to_string(data));

    Ok(())
}

/// When receive a message type subscribe from subscription.
/// subscribe: means that we successfully subscribed to the channel given as the second element in
/// the reply. The third argument represents the number of channels we are currently subscribed to.
fn manage_subscription_message_type_subscribe(
    channel: &str,
    data: &RedisValue,
    logger: &slog::Logger,
) -> Result<(), RedisError> {
    let num = match data {
        RedisValue::Integer(d) => d,
        e => {
            return Err(RedisError::from_message(&format!(
                "The third argument of subscribe message need to be integer. Currently: {:?}.",
                e
            )))
        }
    };

    info!(
        logger,
        "Subscribe successfully of '{}' channel. Currently we are currently subscribe to {} channel(s).",
        channel, num);

    Ok(())
}

/// Watch sentinel and send data to Redis or client.
///
/// Get first sentinel
/// Get master
///    +-> Connect to master
/// Subscribe
///
/// Loop
///   |  check if message from channel
///   |    +-> master change connect to master
///   |
///   |  if channel close look next sentinel until found available or stop if no sentinel available
///   |
///   |  copy data from client to master
///   |    +-> if data start send with old master, send error message to client
///   |
/// End loop
pub fn watch_sentinel(config: &Config, logger: &slog::Logger) -> Result<(), RedisError> {
    let mut sentinel = config.sentinels.as_ref().unwrap();

    // TODO check if sentinel list is empty
    let redis_sentinel_addr = sentinel.get(0).unwrap();
    debug!(
        logger,
        "Create network connection to redis sentinel: {}", &redis_sentinel_addr
    );
    let mut stream = create_redis_connection(redis_sentinel_addr)?;
    let mut redis_connector = RedisConnector::new(&mut stream);

    // Connect to master
    let master_addr = redis_connector.get_master_addr(&config.group_name)?;
    debug!(
        logger,
        "Create network connection to redis master: {}", &master_addr
    );
    let master_connection = create_master_connection(&master_addr)?;

    // Subscribe to Sentinel to notify when master change
    let mut sentinel_subscription = redis_connector.subscribe("+switch-master")?;

    loop {
        match sentinel_subscription.pool() {
            Ok(data) => manage_subscription_data(data, logger)?,
            Err(e) => manage_redis_subscription_error(e)?,
        };
    }
}
