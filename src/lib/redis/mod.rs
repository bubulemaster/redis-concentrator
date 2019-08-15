//! This module contain basic Redis commands.
//!
mod parser;
pub mod stream;
pub mod subscription;
pub mod types;

use crate::lib::redis::stream::RedisStream;
use crate::lib::redis::parser::{read_strict_string, read_bulk_string};
use crate::lib::redis::subscription::RedisSubscription;
use crate::lib::redis::types::RedisError;

pub struct RedisConnector<'a> {
    stream: &'a mut RedisStream
}

impl<'a>  RedisConnector<'a>  {
    pub fn new(stream: &'a mut RedisStream) -> RedisConnector {
        RedisConnector {
            stream
        }
    }

    /// Send PING command and wait PONG response.
    #[allow(dead_code)]
    pub fn ping(&mut self) -> Result<(), RedisError> {
        let cmd = "PING\r\n".as_bytes();

        if let Err(e) = self.stream.write(cmd) {
            return Err(RedisError::from_io_error(e));
        }

        let response = read_strict_string(self.stream)?;

        match response.as_str() {
            "PONG" => Ok(()),
            e => Err(RedisError::from_message(&format!("Invalid ping response : {}", e)))
        }
    }

    /// Subscribe to channel.
    /// Warning, this is blocking method.
    #[allow(dead_code)]
    pub fn subscribe(&mut self, channel: &str) -> Result<RedisSubscription, RedisError> {
        let cmd = format!("SUBSCRIBE {}\r\n", channel);
        let cmd = cmd.as_bytes();

        if let Err(e) = self.stream.write(cmd) {
            return Err(RedisError::from_io_error(e));
        }

        Ok(RedisSubscription::new(self.stream, String::from(channel)))
    }

    /// Get bulk string.
    #[allow(dead_code)]
    pub fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>, RedisError> {
        let key = format!("GET {}\r\n", key);

        if let Err(e) = self.stream.write(key.as_bytes()) {
            return Err(RedisError::from_io_error(e));
        }

        read_bulk_string(self.stream)
    }

    /// Get string
    #[allow(dead_code)]
    pub fn get_string(&mut self, key: &str) -> Result<Option<String>, RedisError> {
        let data = self.get(key)?;

        if let Some(data) = data {
            return match std::str::from_utf8(&data) {
                Ok(v) => Ok(Some(String::from(v))),
                Err(e) => Err(RedisError::from_message(&format!("Invalid UTF-8 sequence: {}", e)))
            };
        }

        Ok(None)
    }
}