//! This module contains struture of connection
//!
use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::subscription::RedisSubscription;
use crate::lib::redis::types::{ErrorKind, RedisError, RedisValue};
use crate::lib::redis::RedisConnector;
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

/// Create redis_subscription to switch master.
fn create_redis_subscription_switch_master(
    redis_sentinel_addr: &str,
) -> Result<RedisSubscription, RedisError> {
    // Create new sentinel connection for subscribe.
    let sentinel_stream = create_redis_connection(redis_sentinel_addr)?;
    // Subscribe to Sentinel to notify when master change
    let mut sentinel_subscription =
        RedisSubscription::new(Box::new(sentinel_stream), String::from("+switch-master"));

    sentinel_subscription.subscribe()?;

    Ok(sentinel_subscription)
}

/// Redis callback.
pub trait RedisBoxConnectorCallback {
    /// Call when change master address.
    fn change_master(
        &mut self,
        group_name: &str,
        old_master_ip: String,
        old_master_port: u16,
        new_master_ip: String,
        new_master_port: u16,
    ) -> Result<(), RedisError>;
}

/// Redis box connector.
pub struct RedisBoxConnector<'a> {
    /// Logger.
    logger: &'a slog::Logger,
    /// Redis master stream.
    master_stream: TcpStream,
    /// Redis subscription
    sentinel_subscription: RedisSubscription,
    /// Redis group name.
    group_name: String,
    /// List of sentinel.
    sentinel_list: Vec<String>, // TODO don't remove in list but create struct with last try
    /// Current sentinel address.
    redis_sentinel_addr: String,
}

impl<'a> RedisBoxConnectorCallback for RedisBoxConnector<'a> {
    fn change_master(
        &mut self,
        group_name: &str,
        old_master_ip: String,
        old_master_port: u16,
        new_master_ip: String,
        new_master_port: u16,
    ) -> Result<(), RedisError> {
        if self.group_name.as_str() != group_name {
            return Ok(());
        }

        debug!(
            self.logger,
            "Receive change master notification Old: {}:{} / New: {}:{} for {}",
            old_master_ip,
            old_master_port,
            new_master_ip,
            new_master_port,
            group_name
        );

        warn!(
            self.logger,
            "Switch to new master {}:{}", new_master_ip, new_master_port
        );

        self.master_stream =
            create_master_connection(&format!("{}:{}", new_master_ip, new_master_port))?;

        Ok(())
    }
}

impl<'a> RedisBoxConnector<'a> {
    pub fn new(
        mut sentinel_list: Vec<String>,
        group_name: &'a str,
        logger: &'a slog::Logger,
    ) -> Result<RedisBoxConnector<'a>, RedisError> {
        let redis_sentinel_addr = sentinel_list.remove(0);

        let sentinel_stream = create_redis_connection(&redis_sentinel_addr)?;
        let mut sentinel_connector = RedisConnector::new(Box::new(sentinel_stream));

        let redis_master_addr = sentinel_connector.get_master_addr(group_name)?;

        debug!(
            logger,
            "Create network connection to redis master: {}", &redis_master_addr
        );

        let master_stream = create_master_connection(&redis_master_addr)?;

        let sentinel_subscription = create_redis_subscription_switch_master(&redis_sentinel_addr)?;

        Ok(RedisBoxConnector {
            logger,
            master_stream,
            sentinel_subscription,
            group_name: String::from(group_name),
            sentinel_list,
            redis_sentinel_addr,
        })
    }

    /// Reconnect to next sentinel.
    fn reconnect_to_next_sentinel(&mut self) -> Result<(), RedisError> {
        if self.sentinel_list.len() == 0 {
            return Err(RedisError::from_message("No more sentinel available!"));
        }

        warn!(
            self.logger,
            "Lost connection with sentinel {}!", &self.redis_sentinel_addr
        );

        // Connect to another sentinel
        self.redis_sentinel_addr = self.sentinel_list.remove(0);

        info!(
            self.logger,
            "Connect to new sentinel {}.", &self.redis_sentinel_addr
        );

        self.sentinel_subscription =
            create_redis_subscription_switch_master(&self.redis_sentinel_addr)?;

        Ok(())
    }

    /// Pool data only from sentinel and reconnect to another sentinel if connection lost.
    pub fn pool_data_from_sentinel(&mut self) -> Result<RedisValue, RedisError> {
        match self.sentinel_subscription.pool() {
            Ok(value) => Ok(value),
            Err(e) => match e.kind() {
                ErrorKind::IoError => match e.io_error_kind().unwrap() {
                    std::io::ErrorKind::BrokenPipe => {
                        self.reconnect_to_next_sentinel()?;

                        Err(RedisError::from_no_data())
                    }
                    _ => Err(e),
                },
                _ => Err(e),
            },
        }
    }

    /// Pool data.
    pub fn pool_data_from_redis_client_and_server(&mut self) -> Result<(), RedisError> {
        // TODO
        Ok(())
    }
}
