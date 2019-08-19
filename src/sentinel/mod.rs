//! This module contain routine to watch sentinels.
//!
use crate::config::Config;
use crate::lib::redis::RedisConnector;
use std::net::TcpStream;
use crate::lib::redis::types::RedisError;
use crate::lib::redis::stream::network::NetworkStream;

/// Create a redis connector to sentinel.
fn create_redis_connector(address: &str) -> Result<RedisConnector, RedisError> {
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

    let mut stream = NetworkStream::new(tcp_stream);
    Ok(RedisConnector::new(&mut stream))
}

/// Watch sentinel and send data to Redis or client.
pub fn watch_sentinel(config: &Config) {
    let mut sentinel = config.sentinels.unwrap();

    // TODO check if sentinel list is empty
    let mut redis_connector = create_redis_connector(sentinel.get(0).unwrap());

    // Get first sentinel
    // Get master
    //   +-> Connect to master
    // Subscribe

    // Loop
    //  |  check if message from channel
    //  |    +-> master change connect to master
    //  |
    //  |  if channel close look next sentinel until found available or stop if no sentinel available
    //  |
    //  |  copy data from client to master
    //  |    +-> if data start send with old master, send error message to client
    //  |
    // End loop
}