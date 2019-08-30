//! This module contain basic Redis commands.
//!
mod parser;
pub mod stream;
pub mod subscription;
pub mod types;

use crate::lib::redis::parser::{read_array, read_bulk_string, read_strict_string};
use crate::lib::redis::stream::RedisStream;
use crate::lib::redis::subscription::RedisSubscription;
use crate::lib::redis::types::{RedisError, RedisValue};

pub struct RedisConnector<'a> {
    stream: &'a mut dyn RedisStream,
}

impl<'a> RedisConnector<'a> {
    pub fn new(stream: &'a mut dyn RedisStream) -> RedisConnector {
        RedisConnector { stream }
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
            e => Err(RedisError::from_message(&format!(
                "Invalid ping response : {}",
                e
            ))),
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
                Err(e) => Err(RedisError::from_message(&format!(
                    "Invalid UTF-8 sequence: {}",
                    e
                ))),
            };
        }

        Ok(None)
    }

    /// Get master addr
    pub fn get_master_addr(&mut self, master_name: &str) -> Result<String, RedisError> {
        let cmd = format!("SENTINEL GET-MASTER-ADDR-BY-NAME {}\r\n", master_name);

        if let Err(e) = self.stream.write(cmd.as_bytes()) {
            return Err(RedisError::from_io_error(e));
        }

        let data = read_array(self.stream)?;

        match data {
            RedisValue::Array(d) => {
                let addr = convert_to_string(d.get(0).unwrap())?;
                let port = convert_to_string(d.get(1).unwrap())?;

                Ok(String::from(format!("{}:{}", addr, port)))
            }
            _ => Err(RedisError::from_message(
                "Impossible, get_master_addr don't return array!",
            )),
        }
    }
}

/// Convert string or return error.
pub fn convert_to_string(value: &RedisValue) -> Result<String, RedisError> {
    match value {
        RedisValue::BulkString(s) => Ok(String::from_utf8_lossy(s).to_string()),
        e => Err(RedisError::from_message(&format!(
            "{:?} is not a BulkString!",
            e
        ))),
    }
}

/// Convert string or return error.
pub fn convert_to_integer(value: &RedisValue) -> Result<isize, RedisError> {
    match value {
        RedisValue::Integer(s) => Ok(s.clone()),
        e => Err(RedisError::from_message(&format!(
            "{:?} is not a Integer!",
            e
        ))),
    }
}
