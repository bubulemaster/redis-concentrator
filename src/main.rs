#[macro_use]
extern crate serde_derive;
extern crate log;
extern crate serde_json;
#[macro_use]
extern crate slog;

mod app;
mod client;
mod config;
mod redis;
mod logging;

use std::env;

use crate::client::{copy_data_from_client_to_redis, watch_new_client_connection};
use crate::config::{get_config, Config};
use crate::logging::build_log;
use crate::redis::sentinel::watch_sentinel;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

struct InitSentinelData {
    tx_main_loop_message: Sender<app::MainLoopEvent>,
    rx_main_loop_message: Receiver<app::MainLoopEvent>,
    redis_master_address: String
}

fn help() {
    println!("RedConcentrator {}", VERSION.unwrap_or("unknown"));
    println!();
    println!("Usage: rsl config-file");
    println!();
}

fn logo(logger: &slog::Logger) {
    // Thanks to http://patorjk.com/software/taag/#p=display&f=Doom&t=Red%20Concentrator
    info!(logger, r"______         _   _____                            _             _             ");
    info!(logger, r"| ___ \       | | /  __ \                          | |           | |            ");
    info!(logger, r"| |_/ /___  __| | | /  \/ ___  _ __   ___ ___ _ __ | |_ _ __ __ _| |_ ___  _ __ ");
    info!(logger, r"|    // _ \/ _` | | |    / _ \| '_ \ / __/ _ \ '_ \| __| '__/ _` | __/ _ \| '__|");
    info!(logger, r"| |\ \  __/ (_| | | \__/\ (_) | | | | (_|  __/ | | | |_| | | (_| | || (_) | |   ");
    info!(logger, r"\_| \_\___|\__,_|  \____/\___/|_| |_|\___\___|_| |_|\__|_|  \__,_|\__\___/|_|   ");
    info!(logger, "");
}

fn run_watch(
    config: &Config,
    logger: slog::Logger,
    tx_main_loop_message: Sender<app::MainLoopEvent>,
    rx_main_loop_message: Receiver<app::MainLoopEvent>,
    redis_master_address: String) -> Result<(), String> {
    debug!(logger, "Receive first master change notification. Start all thread of RedConcentrator");

    if let Err(e) = watch_new_client_connection(&config, logger.clone(), tx_main_loop_message.clone()) {
        return Err(format!("Error from listen client: {:?}", e));
    }

    // TODO master change message
    app::run_main_loop(rx_main_loop_message, redis_master_address, logger);

    // if let Err(e) = copy_data_from_client_to_redis(
    //     &data.new,
    //     logger_redis_master,
    //     rx_master_change,
    //     rx_new_client,
    // ) {
    //     return Err(format!("Error when copy data: {:?}", e));
    // }
    Ok(())
}

/// Run watch sentinel, client.
fn run_watch_sentinel(
    config: &Config,
    logger_redis_sentinel: slog::Logger,
) -> Result<InitSentinelData, String> {
    // Channel to main loop
    let (tx_main_loop_message, rx_main_loop_message): (
        Sender<app::MainLoopEvent>,
        Receiver<app::MainLoopEvent>,
    ) = mpsc::channel();

    if let Err(e) = watch_sentinel(&config, logger_redis_sentinel, tx_main_loop_message.clone()) {
        return Err(format!("Error when running: {:?}", e));
    }

    // Wait master addr.
    let redis_master_address = match rx_main_loop_message.recv() {
        Ok(event) => {            
            if let Some(data) = event.master_change {
                data.new
            } else {
                return Err(
                    String::from("An event raise before master init. It's impossible!!!!")
                );
            }
        }
        Err(e) => {
            return Err(format!(
                "Cannot create first connection to Redis Master: {:?}",
                e
            ));
        }
    };

    Ok(InitSentinelData {
        tx_main_loop_message,
        rx_main_loop_message,
        redis_master_address
    })
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
        match run_watch_sentinel(
            &config,
            logger_main.clone()
        ) {
            Ok(sentinel_data) => {
                if let Err(e) = run_watch(
                    &config,
                    logger_main.clone(),
                    sentinel_data.tx_main_loop_message,
                    sentinel_data.rx_main_loop_message,
                    sentinel_data.redis_master_address) {
                        error!(logger_main.clone(), "{}", e);
                        eprintln!("{}", e);
                        std::process::exit(-1);
                    }
            },
            Err(e) => {
                error!(logger_main.clone(), "{}", e);
                eprintln!("{}", e);
                std::process::exit(-1);
            }
        }
    } else {
        error!(logger_main, "No sentinels found in config file");
    }
}
