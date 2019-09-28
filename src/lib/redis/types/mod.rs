//! This module contain all type of Redis Concentrator.
//!

/// Redis type returned by protocol.
pub const REDIS_TYPE_INTEGER: u8 = b':';
pub const REDIS_TYPE_STRING: u8 = b'+';
pub const REDIS_TYPE_ERROR: u8 = b'-';
pub const REDIS_TYPE_BULK_STRING: u8 = b'$';
pub const REDIS_TYPE_ARRAY: u8 = b'*';

/// Redis value get from redis.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RedisValue {
    /// Redis can return null string and null array.
    Nil,
    Integer(isize),
    String(String),
    BulkString(Vec<u8>),
    Array(Vec<RedisValue>),
}

/// An enum of all error kinds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ErrorKind {
    /// The server generated an invalid response.
    ResponseError,
    /// A script execution was aborted.
    ExecAbortError,
    /// The server cannot response because it's loading a dump.
    BusyLoadingError,
    /// A script that was requested does not actually exist.
    NoScriptError,
    /// This kind is returned if network error.
    IoError,
    /// An error not directly return by Redis.
    OtherError,
    /// If no data available on socket.
    NoDataAvailable,
}

/// Error when call Redis.
#[derive(Debug)]
pub struct RedisError {
    /// Io error.
    io_error: Option<std::io::Error>,
    /// Kind of error
    kind: ErrorKind,
    /// Message
    message: Option<String>,
}

/// Redis error.
impl RedisError {
    /// From std::io::Error
    pub fn from_io_error(e: std::io::Error) -> Self {
        RedisError {
            io_error: Some(e),
            message: None,
            kind: ErrorKind::IoError,
        }
    }

    /// Generic error.
    pub fn from_no_data() -> Self {
        RedisError {
            io_error: None,
            message: Some(String::from("No data available!")),
            kind: ErrorKind::NoDataAvailable,
        }
    }

    /// Generic error.
    pub fn from_message(e: &str) -> Self {
        RedisError {
            io_error: None,
            message: Some(String::from(e)),
            kind: ErrorKind::OtherError,
        }
    }

    /// From error return by Redis.
    pub fn from_redis(code: &str, message: &str) -> Self {
        let kind = match code {
            "ERR" => ErrorKind::ResponseError,
            "EXECABORT" => ErrorKind::ExecAbortError,
            "LOADING" => ErrorKind::BusyLoadingError,
            "NOSCRIPT" => ErrorKind::NoScriptError,
            _ => ErrorKind::OtherError,
        };

        RedisError {
            io_error: None,
            message: Some(String::from(message)),
            kind,
        }
    }

    /// Return kind of error.
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }

    /// Return kind of std::io::Error.
    pub fn io_error_kind(&self) -> Option<std::io::ErrorKind> {
        match self.io_error.as_ref() {
            Some(e) => Some(e.kind().clone()),
            None => None,
        }
    }

    /// Return message if set.
    pub fn message(&self) -> String {
        match self.message.as_ref() {
            Some(s) => s.clone(),
            None => String::new(),
        }
    }
}

impl std::fmt::Display for RedisError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            ErrorKind::IoError => write!(fmt, "IoError: {}", &self.io_error.as_ref().unwrap()),
            ErrorKind::ResponseError => write!(
                fmt,
                "Redis response error: {}",
                self.message.as_ref().unwrap()
            ),
            ErrorKind::ExecAbortError => write!(
                fmt,
                "Redis execution abort: {}",
                self.message.as_ref().unwrap()
            ),
            ErrorKind::BusyLoadingError => write!(
                fmt,
                "Redis busy loading error: {}",
                self.message.as_ref().unwrap()
            ),
            ErrorKind::NoScriptError => write!(
                fmt,
                "Redis no script error: {}",
                self.message.as_ref().unwrap()
            ),
            ErrorKind::OtherError => write!(
                fmt,
                "Error (not Redis error): {}",
                self.message.as_ref().unwrap()
            ),
            ErrorKind::NoDataAvailable => write!(fmt, "Error: {}", self.message.as_ref().unwrap()),
        }
    }
}
