//! This module contain routine to watch sentinels.
//!
use crate::config::Config;

pub fn watch_sentinel(config: &Config) {
    // Get first sentinel
    // Get master
    //   +-> Connect to master
    // Subscribe

    // Loop
    //  |  check if message from channel
    //  |    +-> master change connect to master
    //  |
    //  |  if channel close look next sentinel until found available or stop if no sentinel available
    //  |
    //  |  copy data from client to master
    //  |    +-> if data start send with old master, send error message to client
    //  |
    // End loop
}