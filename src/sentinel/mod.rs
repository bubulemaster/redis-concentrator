//! This module contains routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::convert_to_string;
use crate::lib::redis::types::{ErrorKind, RedisError, RedisValue};
use crate::sentinel::box_connector::{RedisBoxConnector, RedisBoxConnectorCallback};

mod box_connector;

/// Check if no data available.
fn manage_redis_subscription_error(error: RedisError) -> Result<(), RedisError> {
    match error.kind() {
        ErrorKind::NoDataAvailable => return Ok(()),
        _ => return Err(error),
    }
}

/// If we receive message.
fn manage_subscription_data(
    data: RedisValue,
    logger: &slog::Logger,
    callback: &mut dyn RedisBoxConnectorCallback,
) -> Result<(), RedisError> {
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

            manage_subscription_message(&msg_type, &channel, &data, logger, callback)
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
    callback: &mut dyn RedisBoxConnectorCallback,
) -> Result<(), RedisError> {
    match msg_type {
        "subscribe" => manage_subscription_message_type_subscribe(channel, data, logger),
        "message" => manage_subscription_message_type_message(channel, data, logger, callback),
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
    callback: &mut dyn RedisBoxConnectorCallback,
) -> Result<(), RedisError> {
    if channel != "+switch-master" {
        return Ok(());
    }

    /*
    1) "message"
    2) "+switch-master"
    3) "cluster_1 127.0.0.1 6001 127.0.0.1 6000" Groupe name : Old master -> New master
    */
    let message = convert_to_string(data)?;
    debug!(logger, "{:?}", message);

    let split = message.split(' ');
    let vec = split.collect::<Vec<&str>>();

    let group_name = vec.get(0).unwrap();
    let old_master_ip = String::from(*vec.get(1).unwrap());
    let old_master_port = vec.get(2).unwrap().parse().unwrap();
    let new_master_ip = String::from(*vec.get(3).unwrap());
    let new_master_port = vec.get(4).unwrap().parse().unwrap();

    callback.change_master(
        group_name,
        old_master_ip,
        old_master_port,
        new_master_ip,
        new_master_port,
    )
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
    let sentinels_list = config.sentinels.as_ref().unwrap().clone();

    // TODO check if sentinel list is empty

    let mut redis_box = RedisBoxConnector::new(sentinels_list, &config.group_name, logger)?;

    // TODO create thread that bind and listen incomming connection.
    // then set non blocking mode
    // then add this connection to redis_boc

    loop {
        match redis_box.pool_data_from_sentinel() {
            Ok(data) => manage_subscription_data(data, logger, &mut redis_box)?,
            Err(e) => manage_redis_subscription_error(e)?,
        };

        if let Err(e) = redis_box.pool_data_from_redis_client_and_server() {
            // TODO manage error
        }
    }
}
