//! This module contains struture of connection
//!
use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::subscription::RedisSubscription;
use crate::lib::redis::types::{RedisError, RedisValue};
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

/// Redis callback.
pub trait RedisBoxConnectorCallback {
    fn change_master(&mut self);
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
    //group_name: String,
    /// Current Redis master.
    redis_master_addr: String,
}

impl<'a> RedisBoxConnectorCallback for RedisBoxConnector<'a> {
    fn change_master(&mut self) {}
}

impl<'a> RedisBoxConnector<'a> {
    pub fn new(
        redis_sentinel_addr: &'a str,
        group_name: &'a str,
        logger: &'a slog::Logger,
    ) -> Result<RedisBoxConnector<'a>, RedisError> {
        let sentinel_stream = create_redis_connection(redis_sentinel_addr)?;
        let mut sentinel_connector = RedisConnector::new(Box::new(sentinel_stream));

        let redis_master_addr = sentinel_connector.get_master_addr(group_name)?;

        debug!(
            logger,
            "Create network connection to redis master: {}", &redis_master_addr
        );

        let master_stream = create_master_connection(&redis_master_addr)?;

        // Create new sentinel connection for subscribe.
        let sentinel_stream = create_redis_connection(redis_sentinel_addr)?;
        // Subscribe to Sentinel to notify when master change
        let mut sentinel_subscription =
            RedisSubscription::new(Box::new(sentinel_stream), String::from("+switch-master"));

        sentinel_subscription.subscribe()?;

        Ok(RedisBoxConnector {
            logger,
            master_stream,
            sentinel_subscription,
            //group_name: None,
            redis_master_addr,
        })
    }
    /*
        pub fn connect(&'a mut self, group_name: String) -> Result<(), RedisError> {
            self.group_name = Some(group_name);

            //self.sentinel_connector = Some(RedisConnector::new(&mut self.sentinel_stream));

            let mut a = self.sentinel_connector.as_ref().unwrap();

            let b = a.get_master_addr(self.group_name.as_ref().unwrap())?;

            // Connect to master
            self.redis_master_addr = Some(b);

            debug!(
                self.logger,
                "Create network connection to redis master: {}",
                self.redis_master_addr.as_ref().unwrap()
            );

            self.master_stream = Some(create_master_connection(
                self.redis_master_addr.as_ref().unwrap(),
            )?);

            // Subscribe to Sentinel to notify when master change
            self.sentinel_subscription = Some(
                self.sentinel_connector
                    .as_ref()
                    .unwrap()
                    .subscribe("+switch-master")?,
            );

            Ok(())
        }
    */
    /// Pool data.
    pub fn pool(&mut self) -> Result<RedisValue, RedisError> {
        self.sentinel_subscription.pool()
    }
}
