//! This module contain routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::RedisConnector;
use std::net::TcpStream;
use crate::lib::redis::types::RedisError;
use crate::lib::redis::stream::network::NetworkStream;

/// Create a redis connector to sentinel.
fn create_redis_connection(address: &str) -> Result<NetworkStream, RedisError> {
    let tcp_stream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(e) => return Err(RedisError::from_io_error(e))
    };

    // TODO allow to set value cause WouldBlock use it.
    let timeout = std::time::Duration::from_millis(200);

    if let Err(e) = tcp_stream.set_read_timeout(Some(timeout)) {
        return Err(RedisError::from_io_error(e))
    }

    if let Err(e) = tcp_stream.set_nonblocking(false) {
        return Err(RedisError::from_io_error(e))
    }

    Ok(NetworkStream::new(tcp_stream))
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
pub fn watch_sentinel(config: &Config) -> Result<(), RedisError>{
    let mut sentinel = config.sentinels.as_ref().unwrap();

    // TODO check if sentinel list is empty
    let mut stream = create_redis_connection(sentinel.get(0).unwrap())?;
    let mut redis_connector = RedisConnector::new(&mut stream);

    let redis_group_name = redis_connector.get_master_add(&config.group_name)?;

    println!(">>: {}", redis_group_name);

    Ok(())
}