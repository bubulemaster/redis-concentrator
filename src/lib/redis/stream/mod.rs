//! This module contain abstract type of stream.
//!

pub mod network;

/// Abstract stream for redis.
pub trait RedisStream {
    // Write data on lib.redis.stream.network.
    fn write(&mut self, data: &[u8]) ->  std::io::Result<()>;

    /// Get byte from stream.
    fn get(&mut self) -> std::io::Result<Option<u8>>;

    /// Get X byte from stream.
    fn get_data(&mut self, size: usize) -> std::io::Result<Vec<u8>>;

    /// Search in stream pattern and return data until pattern (pattern included).
    fn get_until(&mut self, pattern: &[u8]) -> std::io::Result<Vec<u8>>;
}