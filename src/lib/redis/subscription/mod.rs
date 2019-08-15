//! This module contain async subscription structure.
//!

use std::fmt::{Debug, Formatter, Error};
use crate::lib::redis::stream::RedisStream;
use crate::lib::redis::types::{RedisError, RedisValue};
use crate::lib::redis::parser::read_array;

/// Structure when you subscribe to channel.
pub struct RedisSubscription<'a> {
    stream: &'a mut RedisStream,
    channel: String
}

impl<'a> RedisSubscription<'a> {
    pub fn new(stream: &'a mut RedisStream, channel: String) -> RedisSubscription {
        RedisSubscription {
            stream,
            channel
        }
    }

    /// Pool new message.
    pub fn pool(&mut self) -> Result<RedisValue, RedisError> {
        read_array(self.stream)
    }
}

impl<'a> Debug for RedisSubscription<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "RedisSubscription(channel='{}')", &self.channel)
    }
}