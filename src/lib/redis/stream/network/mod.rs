//! This module contain abstract type of network.
//!

use std::net::TcpStream;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use crate::lib::redis::stream::RedisStream;

const BUFFER_SIZE: usize = 2048;

/// Abstract stream for network.
/// Read automatically data if need.
pub struct NetworkStream {
    /// Network stream.
    stream: TcpStream,
    /// Internal buffer.
    pub buf: Vec<u8>,
}

impl NetworkStream {
    pub fn new(stream: TcpStream) -> NetworkStream {
        NetworkStream {
            stream,
            buf: Vec::with_capacity(BUFFER_SIZE)
        }
    }

    /// Read data from lib.redis.stream.network and update buffer size.
    fn read(&mut self) -> std::io::Result<()> {
        let mut buf = [0; BUFFER_SIZE];

        match self.stream.read(&mut buf) {
            Ok(len) =>
                if len == 0 {
                    // If 0, that mean socket close other raise WouldBlock.
                    Err(std::io::Error::new(ErrorKind::BrokenPipe, "Server close socket"))
                } else {
                    self.buf.extend_from_slice(&buf[0..len]);
                    Ok(())
                },
            Err(e) =>
                if e.kind() != ErrorKind::WouldBlock {
                    Err(e)
                } else {
                    // In case of WouldBlock, no data available.
                    Ok(())
                }
        }
    }

    /// Search something in current buffer.
    fn search_in_buffer(&self, start: usize, pattern: &[u8]) -> Option<usize> {
        let mut pattern_index = 0;
        let end = self.buf.len();

        for index in start..end {
            match self.buf.get(index) {
                Some(c) => {
                    if *c == pattern[pattern_index] {
                        // That match, check next char
                        pattern_index = pattern_index + 1;

                        // Out of pattern, that means we find it !
                        if pattern_index >= pattern.len() {
                            return Some(start + index);
                        }
                    } else {
                        pattern_index = 0;
                    }
                },
                None => return None
            }
        }

        None
    }

}

impl RedisStream for NetworkStream {
    fn write(&mut self, data: &[u8]) ->  std::io::Result<()> {
        match self.stream.write(data) {
            Ok(len) =>
                if len != data.len() {
                    Err(
                        std::io::Error::new(
                            ErrorKind::WriteZero,
                            format!("Write only {} bytes on {}", len, data.len())))
                } else {
                    Ok(())
                },
            Err(e) => Err(e)
        }
    }

    fn get(&mut self) -> std::io::Result<Option<u8>> {
        if self.buf.is_empty() {
            self.read()?;
        }

        if self.buf.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.buf.remove(0)))
        }
    }

    fn get_data(&mut self, size: usize) -> std::io::Result<Vec<u8>> {
        let mut size = size;

        if self.buf.len() < size {
            self.read()?;
        }

        if self.buf.len() < size {
            // Always greater, truncate
            size = self.buf.len();
        }

        // Split of extract data from size to end.
        // But this is our new buffer stream.
        let new_buf = self.buf.split_off(size);

        // That why, we clone the new buf that is what we want return.
        let ret_buf = self.buf.clone();

        // Clear ou buffer stream.
        self.buf.clear();

        // And add data.
        self.buf.extend(new_buf);

        Ok(ret_buf)
    }

    fn get_until(&mut self, pattern: &[u8]) -> std::io::Result<Vec<u8>> {
        let mut index = 0;

        loop {
            match self.search_in_buffer(index, pattern) {
                // pos = 2 pattern = '\r\n'
                // That mean pos is '\r' we must substract 1 to have right size
                Some(pos) => return self.get_data(pos + pattern.len() - 1),
                None => {
                    let old_buf_size = self.buf.len();

                    // Try to get new data
                    self.read()?;

                    if old_buf_size == self.buf.len() {
                        // No more data read, stop search.
                        return Ok(Vec::new())
                    } else {
                        index = old_buf_size;
                    }
                }
            }
        }
    }
}