//! This module contain abstract type of network.
//!

use crate::lib::redis::stream::RedisStream;
use std::io::ErrorKind;

/// Abstract stream for network.
/// Read automatically data if need.
pub struct TestRedisStream {
    /// Internal buffer.
    read_buf: Vec<u8>,
    pub write_buf: Vec<u8>,
}

impl TestRedisStream {
    pub fn new(buf: Vec<u8>) -> Self {
        TestRedisStream {
            read_buf: buf,
            write_buf: Vec::new(),
        }
    }

    /// Search something in current buffer.
    fn search_in_buffer(&self, start: usize, pattern: &[u8]) -> Option<usize> {
        let mut pattern_index = 0;
        let end = self.read_buf.len();

        for index in start..end {
            match self.read_buf.get(index) {
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
                }
                None => return None,
            }
        }

        None
    }
}

impl RedisStream for TestRedisStream {
    fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.write_buf.extend_from_slice(data);

        Ok(())
    }

    fn get(&mut self) -> std::io::Result<Option<u8>> {
        if self.read_buf.is_empty() {
            return Err(std::io::Error::new(
                ErrorKind::BrokenPipe,
                "Server close socket",
            ));
        }

        Ok(Some(self.read_buf.remove(0)))
    }

    fn get_data(&mut self, size: usize) -> std::io::Result<Vec<u8>> {
        if self.read_buf.len() < size {
            if self.read_buf.is_empty() {
                return Err(std::io::Error::new(
                    ErrorKind::BrokenPipe,
                    "Server close socket",
                ));
            }
        }

        // Split of extract data from size to end.
        // But this is our new buffer stream.
        let new_buf = self.read_buf.split_off(size);

        // That why, we clone the new buf that is what we want return.
        let ret_buf = self.read_buf.clone();

        // Clear our buffer stream.
        self.read_buf.clear();

        // And add data.
        self.read_buf.extend(new_buf);

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
                    let old_buf_size = self.read_buf.len();

                    if old_buf_size == self.read_buf.len() {
                        // No more data read, stop search.
                        return Ok(Vec::new());
                    } else {
                        index = old_buf_size;
                    }
                }
            }
        }
    }
}
