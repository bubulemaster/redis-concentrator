//! This module contain async subscription structure.
//!
#[cfg(test)]
pub mod tests;

use crate::lib::redis::parser::read_array;
use crate::lib::redis::stream::RedisStream;
use crate::lib::redis::types::{RedisError, RedisValue};
use std::fmt::{Debug, Error, Formatter};

/// Structure when you subscribe to channel.
pub struct RedisSubscription {
    stream: Box<dyn RedisStream>,
    channel: String,
}

impl<'a> RedisSubscription {
    pub fn new(stream: Box<dyn RedisStream>, channel: String) -> Self {
        RedisSubscription { stream, channel }
    }

    /// Start subscription.
    pub fn subscribe(&mut self) -> Result<(), RedisError> {
        let cmd = format!("SUBSCRIBE {}\r\n", &self.channel);
        let cmd = cmd.as_bytes();

        if let Err(e) = self.stream.write(cmd) {
            return Err(RedisError::from_io_error(e));
        }

        Ok(())
    }

    /// Pool new message.
    pub fn pool(&mut self) -> Result<RedisValue, RedisError> {
        read_array(&mut self.stream)
    }
}

impl<'a> Debug for RedisSubscription {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "RedisSubscription(channel='{}')", &self.channel)
    }
}
