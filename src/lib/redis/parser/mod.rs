//! This module contain parse basic routine.
//!

use crate::lib::redis::stream::RedisStream;
use crate::lib::redis::types::REDIS_TYPE_ARRAY;
use crate::lib::redis::types::REDIS_TYPE_BULK_STRING;
use crate::lib::redis::types::REDIS_TYPE_ERROR;
use crate::lib::redis::types::REDIS_TYPE_INTEGER;
use crate::lib::redis::types::{RedisError, RedisValue, REDIS_TYPE_STRING};

#[cfg(test)]
pub mod tests;

/// Redis type get from redis.
#[derive(Debug, PartialEq)]
enum RedisType {
    Integer,
    String,
    BulkString,
    Array,
    Error,
}

/// Get one byte. If none, raise error.
#[inline(always)]
fn get_byte(stream: &mut Box<dyn RedisStream>) -> Result<u8, RedisError> {
    match stream.get() {
        Ok(c) => match c {
            Some(c) => Ok(c),
            None => Err(RedisError::from_no_data()),
        },
        Err(e) => Err(RedisError::from_io_error(e)),
    }
}

/// Return type of data.
fn get_type(data: u8) -> Result<RedisType, RedisError> {
    match data {
        REDIS_TYPE_STRING => Ok(RedisType::String),
        REDIS_TYPE_BULK_STRING => Ok(RedisType::BulkString),
        REDIS_TYPE_ARRAY => Ok(RedisType::Array),
        REDIS_TYPE_ERROR => Ok(RedisType::Error),
        REDIS_TYPE_INTEGER => Ok(RedisType::Integer),
        e => Err(RedisError::from_message(&format!("Unknow type '{}'", e))),
    }
}

/// Read byte until "\r\n" and convert to string.
fn read_string_from_stream(stream: &mut Box<dyn RedisStream>) -> Result<String, RedisError> {
    let data = match stream.get_until("\r\n".as_bytes()) {
        Ok(a) => a,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    let value = unsafe { std::str::from_utf8_unchecked(&data[..data.len() - 2]) };

    Ok(String::from(value))
}

/// Read only array size.
fn read_array_size(stream: &mut Box<dyn RedisStream>) -> Result<isize, RedisError> {
    let header = check_error(stream)?;

    // First char must be '*'
    if header != REDIS_TYPE_ARRAY {
        return Err(RedisError::from_message(&format!(
            "Not an array but a {}",
            what_is(&[header])
        )));
    }

    let size = read_string_from_stream(stream)?;

    match size.parse::<isize>() {
        Ok(i) => Ok(i),
        Err(e) => Err(RedisError::from_message(&format!(
            "Invalid integer: {} in '{}'",
            e, size
        ))),
    }
}

/// Return type of data.
fn what_is(data: &[u8]) -> String {
    match data[0] {
        REDIS_TYPE_STRING => String::from("String"),
        REDIS_TYPE_BULK_STRING => String::from("BulkString"),
        REDIS_TYPE_ARRAY => String::from("Array"),
        REDIS_TYPE_ERROR => String::from("Error"),
        REDIS_TYPE_INTEGER => String::from("Integer"),
        e => format!("Unknow '0x{}'", e),
    }
}

/// Read error message.
fn read_error_from_stream(stream: &mut Box<dyn RedisStream>) -> RedisError {
    let mut message = match read_string_from_stream(stream) {
        Ok(m) => m,
        Err(e) => return e,
    };

    let position = match message.find(' ') {
        Some(p) => p,
        None => return RedisError::from_message(&message),
    };

    let explain = message.split_off(position);

    RedisError::from_redis(&message, &explain)
}

/// Check if message contains error.
/// If no error return Ok(u8) otherwise return Err(RedisError).
/// The u8 is character checked to be error.
fn check_error(stream: &mut Box<dyn RedisStream>) -> Result<u8, RedisError> {
    // First char must be '$'
    let c = get_byte(stream)?;

    if c != REDIS_TYPE_ERROR {
        return Ok(c);
    }

    Err(read_error_from_stream(stream))
}

/// Read byte until "\r\n" and convert to integer.
fn read_integer_from_stream(stream: &mut Box<dyn RedisStream>) -> Result<isize, RedisError> {
    let size = read_string_from_stream(stream)?;

    match size.parse::<isize>() {
        Ok(i) => Ok(i),
        Err(e) => Err(RedisError::from_message(&format!(
            "Invalid integer: {} in '{}'",
            e, size
        ))),
    }
}

/// Read all byte and convert to [u8].
fn read_bulk_string_from_stream(
    stream: &mut Box<dyn RedisStream>,
) -> Result<Option<Vec<u8>>, RedisError> {
    // Get first part: the size
    let size = read_string_from_stream(stream)?;

    let size = match size.parse::<isize>() {
        Ok(i) => i,
        Err(e) => {
            return Err(RedisError::from_message(&format!(
                "Invalid integer: {} in '{}'",
                e, size
            )))
        }
    };

    // Null string
    if size < 0 {
        return Ok(None);
    }

    let data = match stream.get_data(size as usize) {
        Ok(data) => data,
        Err(e) => return Err(RedisError::from_io_error(e)),
    };

    // Skip last '\r\n'
    if let Err(e) = stream.get_data(2) {
        return Err(RedisError::from_io_error(e));
    }

    Ok(Some(data))
}

/// Read an array.
fn read_array_from_stream(
    stream: &mut Box<dyn RedisStream>,
    array_size: usize,
) -> Result<RedisValue, RedisError> {
    let mut result: Vec<RedisValue> = Vec::with_capacity(array_size);

    for _ in 0..array_size {
        let data_type = get_byte(stream)?;
        let data_type = get_type(data_type)?;

        match data_type {
            RedisType::Integer => {
                let s = read_integer_from_stream(stream)?;
                result.push(RedisValue::Integer(s));
            }
            RedisType::BulkString => {
                let s = read_bulk_string_from_stream(stream)?;

                match s {
                    Some(s) => result.push(RedisValue::BulkString(s)),
                    None => result.push(RedisValue::Nil),
                }
            }
            RedisType::String => {
                let s = read_string_from_stream(stream)?;
                result.push(RedisValue::String(s));
            }
            RedisType::Array => {
                let size = read_integer_from_stream(stream)?;

                if size < 0 {
                    result.push(RedisValue::Nil);
                } else {
                    result.push(read_array_from_stream(stream, array_size as usize)?);
                }
            }
            // Normally, never happen
            RedisType::Error => return Err(read_error_from_stream(stream)),
        }
    }

    Ok(RedisValue::Array(result))
}

/// Read strict string, not bulk string.
/// Must contain '\r\n' at end (but not include in result).
pub fn read_strict_string(stream: &mut Box<dyn RedisStream>) -> Result<String, RedisError> {
    let header = check_error(stream)?;

    if header != REDIS_TYPE_STRING {
        return Err(RedisError::from_message(&format!(
            "Not a string but a {}",
            what_is(&[header])
        )));
    }

    read_string_from_stream(stream)
}

/// Read integer value.
pub fn read_integer(stream: &mut Box<dyn RedisStream>) -> Result<isize, RedisError> {
    let header = check_error(stream)?;

    // First char must be ':'
    if header != REDIS_TYPE_INTEGER {
        return Err(RedisError::from_message(&format!(
            "Not an integer but a {}",
            what_is(&[header])
        )));
    }

    read_integer_from_stream(stream)
}

/// Read bulk string.
/// Bulk string can contain non printable char.
#[allow(dead_code)]
pub fn read_bulk_string(stream: &mut Box<dyn RedisStream>) -> Result<Option<Vec<u8>>, RedisError> {
    let header = check_error(stream)?;

    // First char must be '$'
    if header != REDIS_TYPE_BULK_STRING {
        return Err(RedisError::from_message(&format!(
            "Not a bulk string but a {}",
            what_is(&[header])
        )));
    }

    read_bulk_string_from_stream(stream)
}

/// Read an array.
#[allow(dead_code)]
pub fn read_array(stream: &mut Box<dyn RedisStream>) -> Result<RedisValue, RedisError> {
    let array_size = read_array_size(stream)?;

    if array_size < 0 {
        Ok(RedisValue::Nil)
    } else {
        read_array_from_stream(stream, array_size as usize)
    }
}
