extern crate log;
extern crate serde_json;

mod app;
mod client;
mod config;
mod redis;
mod workers;

use std::env;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time;

use app::messages::MainLoopEvent;
use log::{error, info, debug};
use workers::create_workers_pool;

use crate::client::{copy_data_from_client_to_redis, watch_new_client_connection};
use crate::config::{get_config, Config};
use crate::redis::sentinel::watch_sentinel;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

struct InitSentinelData {
    tx_main_loop_message: Sender<MainLoopEvent>,
    rx_main_loop_message: Receiver<MainLoopEvent>,
    redis_master_address: String
}

fn help() {
    println!();
    println!("Usage: rsl config-file");
    println!();
}

fn print_logo() {
    // Thanks to http://patorjk.com/software/taag/#p=display&f=Doom&t=Red%20Concentrator
    info!(r"______         _   _____                            _             _             ");
    info!(r"| ___ \       | | /  __ \                          | |           | |            ");
    info!(r"| |_/ /___  __| | | /  \/ ___  _ __   ___ ___ _ __ | |_ _ __ __ _| |_ ___  _ __ ");
    info!(r"|    // _ \/ _` | | |    / _ \| '_ \ / __/ _ \ '_ \| __| '__/ _` | __/ _ \| '__|");
    info!(r"| |\ \  __/ (_| | | \__/\ (_) | | | | (_|  __/ | | | |_| | | (_| | || (_) | |   ");
    info!(r"\_| \_\___|\__,_|  \____/\___/|_| |_|\___\___|_| |_|\__|_|  \__,_|\__\___/|_|   ");
    info!("");
}

fn run_watch(
    config: &Config,
    tx_main_loop_message: Sender<MainLoopEvent>,
    rx_main_loop_message: Receiver<MainLoopEvent>,
    redis_master_address: String) -> Result<(), String> {
    debug!("Receive first master change notification. Start all thread of RedConcentrator");

    if let Err(e) = watch_new_client_connection(&config, tx_main_loop_message.clone()) {
        return Err(format!("Error from listen client: {:?}", e));
    }

    let workers_map = create_workers_pool(config.workers.pool.min, &tx_main_loop_message);

    // TODO master change message
    if let Err(e) = app::run_main_loop(rx_main_loop_message, redis_master_address, workers_map) {
        return Err(format!("Error run main loop: {:?}", e));
    }

    Ok(())
}

/// Run watch sentinel, client.
fn run_watch_sentinel(
    config: &Config,
) -> Result<InitSentinelData, String> {
    // Channel to main loop
    let (tx_main_loop_message, rx_main_loop_message): (
        Sender<MainLoopEvent>,
        Receiver<MainLoopEvent>,
    ) = mpsc::channel();

    info!("Watch sentinel at startup to get master address");

    if let Err(e) = watch_sentinel(&config, tx_main_loop_message.clone()) {
        return Err(format!("Error when running: {:?}", e));
    }

    info!("Wait to get master address");

    let timeout = time::Duration::from_millis(config.timeout.sentinels.clone());

    // Wait master addr.
    let redis_master_address = match rx_main_loop_message.recv_timeout(timeout) {
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

fn fatal_error(e: String) {
    error!("{}", e);
    eprintln!("{}", e);
    std::process::exit(-1);
}

// TODO return error value
fn main() {
    println!("RedConcentrator {}", VERSION.unwrap_or("unknown"));

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

    if let Err(e) = log4rs::init_file(config.log.file.clone(), Default::default()) {
        eprintln!("Log file error ({}): {}", config.log.file.clone(), e);
        std::process::exit(-1);
    }

    if config.log.logo {
        print_logo();
    }

    if config.sentinels.is_some() {
        match run_watch_sentinel(&config) {
            Ok(sentinel_data) => {
                if let Err(e) = run_watch(
                    &config,
                    sentinel_data.tx_main_loop_message,
                    sentinel_data.rx_main_loop_message,
                    sentinel_data.redis_master_address) {
                        fatal_error(e);
                    }
            },
            Err(e) => fatal_error(e)
        }
    } else {
        error!("No sentinels found in config file");
    }
}
