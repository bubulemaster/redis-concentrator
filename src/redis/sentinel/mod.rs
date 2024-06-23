//! This module contains routine to watch sentinels.
//!
use crate::app::MainLoopEvent;
use crate::config::Config;
use crate::redis::subscription::RedisSubscription;
use crate::redis::types::{ErrorKind, RedisError, RedisValue};
use crate::redis::{convert_to_string, RedisConnector};
use crate::redis::node::{create_redis_stream_connection, create_redis_stream_connection_blocking};
use std::sync::mpsc::Sender;
use std::thread;

/// Struct to communicate a master change ip address.
#[derive(Debug)]
pub struct MasterChangeNotification {
    /// Addresse: "ww.xx.yy.zz:ppppp".
    pub new: String,
    /// Addresse: "ww.xx.yy.zz:ppppp".
    pub old: String,
    /// Name of redis group.
    pub group_name: String,
}

/// Create redis_subscription to switch master.
fn create_redis_subscription_switch_master(
    redis_sentinel_addr: &str,
) -> Result<RedisSubscription, RedisError> {
    // Create new sentinel connection for subscribe.
    let sentinel_stream = create_redis_stream_connection(redis_sentinel_addr)?;
    // Subscribe to Sentinel to notify when master change
    let mut sentinel_subscription =
        RedisSubscription::new(Box::new(sentinel_stream), String::from("+switch-master"));

    sentinel_subscription.subscribe()?;

    Ok(sentinel_subscription)
}

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
    tx_master_change: &Sender<MainLoopEvent>,
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

            manage_subscription_message(&msg_type, &channel, &data, logger, tx_master_change)
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
    tx_master_change: &Sender<MainLoopEvent>,
) -> Result<(), RedisError> {
    match msg_type {
        "subscribe" => manage_subscription_message_type_subscribe(channel, data, logger),
        "message" => {
            manage_subscription_message_type_message(channel, data, logger, tx_master_change)
        }
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
    tx_master_change: &Sender<MainLoopEvent>,
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

    let group_name = *vec.get(0).unwrap();
    let old_master_ip = *vec.get(1).unwrap();
    let old_master_port = *vec.get(2).unwrap();
    let new_master_ip = *vec.get(3).unwrap();
    let new_master_port = *vec.get(4).unwrap();

    let new_master_addr = format!(
        "{}:{}",
        String::from(new_master_ip),
        String::from(new_master_port)
    );
    let old_master_addr = format!(
        "{}:{}",
        String::from(old_master_ip),
        String::from(old_master_port)
    );

    send_notification(
        &new_master_addr,
        &old_master_addr,
        group_name,
        logger,
        tx_master_change,
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

/// Create notification of master change.
fn send_notification(
    new_redis_master_addr: &str,
    old_redis_master_addr: &str,
    group_name: &str,
    _logger: &slog::Logger,
    tx_master_change: &Sender<MainLoopEvent>,
) -> Result<(), RedisError> {
    let msg = MasterChangeNotification {
        new: String::from(new_redis_master_addr),
        old: String::from(old_redis_master_addr),
        group_name: String::from(group_name),
    };

    // TODO check if master role

    tx_master_change.send(MainLoopEvent::master_change(msg)).unwrap();

    Ok(())
}
/// Main loop to watch sentinel.
fn watch_sentinel_loop(
    logger: slog::Logger,
    tx_master_change: Sender<MainLoopEvent>,
    sentinels_list: Vec<String>,
    group_name: String,
) -> Result<(), RedisError> {
    let mut redis_master_addr = String::new();

    // Iterate on sentinel list in case of lost sentinel
    for redis_sentinel_addr in sentinels_list {
        let sentinel_stream = create_redis_stream_connection_blocking(&redis_sentinel_addr)?;
        let mut sentinel_connector = RedisConnector::new(Box::new(sentinel_stream));
        let result_new_redis_master_addr = sentinel_connector.get_master_addr(&group_name);

        if let Err(e) = result_new_redis_master_addr {
            error!(logger, "Master group not found or network connection issue.");
            return Err(e);
        }

        let new_redis_master_addr = result_new_redis_master_addr.unwrap();

        // If master change, create notification.
        if new_redis_master_addr != redis_master_addr {
            send_notification(
                &new_redis_master_addr,
                &redis_master_addr,
                &group_name,
                &logger,
                &tx_master_change,
            )?;

            redis_master_addr = new_redis_master_addr;
        }

        info!(logger, "Connect to new sentinel {}.", &redis_sentinel_addr);

        let mut sentinel_subscription =
            create_redis_subscription_switch_master(&redis_sentinel_addr)?;

        'sentinel_pool: loop {
            match sentinel_subscription.pool() {
                Ok(data) => manage_subscription_data(data, &logger, &tx_master_change)?,
                Err(e) => {
                    if let ErrorKind::IoError = e.kind() {
                        if let std::io::ErrorKind::BrokenPipe = e.io_error_kind().unwrap() {
                            warn!(
                                logger,
                                "Lost connection with sentinel {}!", &redis_sentinel_addr
                            );

                            break 'sentinel_pool;
                        }
                    }

                    manage_redis_subscription_error(e)?;
                }
            };
        }
    }

    Ok(())
}

/// Watch sentinel and send data to Redis or client.
///
/// Get first sentinel
/// Get master
///    +-> Connect to master and check if master
/// Subscribe
///
/// Loop
///   |  check if message from channel
///   |    +-> master change connect to master
///   |
///   |  if channel close look next sentinel until found available or stop if no sentinel available
///   |
/// End loop
pub fn watch_sentinel(
    config: &Config,
    logger: slog::Logger,
    tx_master_change: Sender<MainLoopEvent>,
) -> Result<(), RedisError> {
    let sentinels_list = config.sentinels.as_ref().unwrap().clone();
    let group_name = String::from(&config.group_name);

    if sentinels_list.len() == 0 {
        error!(logger, "Sentinel list empty.");
        return Err(RedisError::from_message("Sentinel list empty."));
    }

    thread::spawn(move || {
        let status = watch_sentinel_loop(logger, tx_master_change, sentinels_list, group_name);

        if let Err(_e) = status {
            // TODO send to main process to stop it
        }
    });

    Ok(())
}
