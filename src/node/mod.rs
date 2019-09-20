//! This module contains routine to connect to redis node.
//!
use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::types::RedisError;
use std::net::TcpStream;

/// Create a a network stream in blocking mode.
pub fn create_redis_stream_connection_blocking(address: &str) -> Result<NetworkStream, RedisError> {
    create_redis_stream_param(address, false)
}

/// Create a a network stream in non blocking mode.
pub fn create_redis_stream_connection(address: &str) -> Result<NetworkStream, RedisError> {
    create_redis_stream_param(address, true)
}

/// Create redis stream.
fn create_redis_stream_param(address: &str, blocking: bool) -> Result<NetworkStream, RedisError> {
    let tcp_stream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    // TODO allow to set value cause WouldBlock use it.
    let timeout = std::time::Duration::from_millis(200);

    if let Err(e) = tcp_stream.set_read_timeout(Some(timeout)) {
        return Err(RedisError::from_io_error(e));
    }

    if !blocking {
        if let Err(e) = tcp_stream.set_nonblocking(blocking) {
            return Err(RedisError::from_io_error(e));
        }
    }

    if let Err(e) = tcp_stream.set_nodelay(true) {
        return Err(RedisError::from_io_error(e));
    }

    Ok(NetworkStream::new(tcp_stream))
}

/// Create a raw tcp connection.
pub fn create_stream_connection(address: &str) -> Result<TcpStream, RedisError> {
    let tcp_stream = match TcpStream::connect(address) {
        Ok(s) => s,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    // TODO allow to set value cause WouldBlock use it.
    let timeout = std::time::Duration::from_millis(200);

    if let Err(e) = tcp_stream.set_read_timeout(Some(timeout)) {
        return Err(RedisError::from_io_error(e));
    }

    if let Err(e) = tcp_stream.set_write_timeout(Some(timeout)) {
        return Err(RedisError::from_io_error(e));
    }

    if let Err(e) = tcp_stream.set_nonblocking(true) {
        return Err(RedisError::from_io_error(e));
    }

    Ok(tcp_stream)
}
