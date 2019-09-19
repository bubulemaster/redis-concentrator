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
mod node;
mod sentinel;

use std::env;

use crate::client::{copy_data_from_client_to_redis, watch_client};
use crate::config::{get_config, Config};
use crate::logging::build_log;
use crate::sentinel::{watch_sentinel, MasterChangeNotification};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

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

/// Run watch sentinel, client.
fn run_watch(
    config: &Config,
    logger_client: slog::Logger,
    logger_redis_sentinel: slog::Logger,
    logger_redis_master: slog::Logger,
    logger_main: slog::Logger,
) -> Result<(), String> {
    // Channel to notify when master change
    let (tx_master_change, rx_master_change): (
        Sender<MasterChangeNotification>,
        Receiver<MasterChangeNotification>,
    ) = mpsc::channel();

    // Channel to notify when new client
    let (tx_new_client, rx_new_client): (
        Sender<(TcpStream, SocketAddr)>,
        Receiver<(TcpStream, SocketAddr)>,
    ) = mpsc::channel();

    if let Err(e) = watch_sentinel(&config, logger_redis_sentinel, tx_master_change) {
        return Err(format!("Error when running: {:?}", e));
    }

    // Wait master addr.
    match rx_master_change.recv() {
        Ok(data) => {
            debug!(logger_main, "Receive first master change notification");

            if let Err(e) = watch_client(&config, logger_client, tx_new_client) {
                return Err(format!("Error from listen client: {:?}", e));
            }

            if let Err(e) = copy_data_from_client_to_redis(
                &data.new,
                logger_redis_master,
                rx_master_change,
                rx_new_client,
            ) {
                return Err(format!("Error when copy data: {:?}", e));
            }
        }
        Err(e) => {
            return Err(format!(
                "Cannot create first connection to Redis Master: {:?}",
                e
            ));
        }
    }

    Ok(())
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
    let logger_main = build_log(&config);

    if config.log.logo {
        logo(&logger_main);
    }

    if config.sentinels.is_some() {
        if let Err(e) = run_watch(
            &config,
            logger_main.clone(),
            logger_main.clone(),
            logger_main.clone(),
            logger_main.clone(),
        ) {
            error!(logger_main, "{}", e);
            eprintln!("{}", e);
            std::process::exit(-1);
        }
    } else {
        error!(logger_main, "No sentinels found in config file");
    }
}
