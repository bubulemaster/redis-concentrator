#[macro_use]
extern crate serde_derive;
extern crate log;
extern crate serde_json;
#[macro_use]
extern crate slog;

mod client;
mod config;
mod lib;
mod logging;
mod sentinel;

use std::env;

use crate::client::watch_client;
use crate::config::{get_config, Config};
use crate::logging::create_log;
use crate::sentinel::{watch_sentinel, MasterChangeNotification};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

fn help() {
    println!("redis-concentrator {}", VERSION.unwrap_or("unknown"));
    println!();
    println!("Usage: redis-concentrator config-file");
    println!();
}

fn logo(logger: &slog::Logger) {
    // Thanks to http://patorjk.com/software/taag/#p=display&f=Doom&t=Red%20Concentrator
    info!(
        logger,
        r"______         _   _____                            _             _             "
    );
    info!(
        logger,
        r"| ___ \       | | /  __ \                          | |           | |            "
    );
    info!(
        logger,
        r"| |_/ /___  __| | | /  \/ ___  _ __   ___ ___ _ __ | |_ _ __ __ _| |_ ___  _ __ "
    );
    info!(
        logger,
        r"|    // _ \/ _` | | |    / _ \| '_ \ / __/ _ \ '_ \| __| '__/ _` | __/ _ \| '__|"
    );
    info!(
        logger,
        r"| |\ \  __/ (_| | | \__/\ (_) | | | | (_|  __/ | | | |_| | | (_| | || (_) | |   "
    );
    info!(
        logger,
        r"\_| \_\___|\__,_|  \____/\___/|_| |_|\___\___|_| |_|\__|_|  \__,_|\__\___/|_|   "
    );
    info!(logger, "");
}

/// Create logger.
fn build_log(config: &Config) -> slog::Logger {
    match create_log(&config) {
        Some(l) => l,
        None => {
            eprintln!("Error: cannot create log!");
            std::process::exit(-1);
        }
    }
}

// TODO return error value
fn main() {
    // Get command line options
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        help();

        std::process::exit(-1);
    }

    let config_file = args[1].clone();

    // We load config file.
    let config = match get_config(config_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: can't read config file: {}", e);
            std::process::exit(-1);
        }
    };

    // Set log
    let logger_client = build_log(&config);
    let logger_redis_sentinel = build_log(&config);
    let logger_redis_master = build_log(&config);
    let logger_main = build_log(&config);

    if config.log.logo {
        logo(&logger_redis_master);
    }

    if config.sentinels.is_some() {
        // Channel to notify when master change
        let (tx_master_change, rx_master_change): (
            Sender<MasterChangeNotification>,
            Receiver<MasterChangeNotification>,
        ) = mpsc::channel();

        // TODO create channel to allow communicate between client connection and sentinel watcher
        if let Err(e) = watch_client(&config, logger_client) {
            error!(logger_main, "Error from listen client : {:?}", e);
        }

        if let Err(e) = watch_sentinel(&config, logger_redis_sentinel, tx_master_change) {
            error!(logger_main, "Error when running : {:?}", e);
        }

        loop {
            let msg = rx_master_change.recv().unwrap(); // TODO throw error if watch sentinel exit
            println!("Master change: {:?}", msg)
            // TODO copy data from client to sentinel
        }
    } else {
        error!(logger_main, "No sentinels found in config file");
    }
}
