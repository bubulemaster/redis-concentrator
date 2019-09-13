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
use crate::config::get_config;
use crate::logging::create_log;
use crate::sentinel::watch_sentinel;

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
    let logger_client = match create_log(&config) {
        Some(l) => l,
        None => {
            eprintln!("Error: cannot create log!");
            std::process::exit(-1);
        }
    };

    let logger_server = match create_log(&config) {
        Some(l) => l,
        None => {
            eprintln!("Error: cannot create log!");
            std::process::exit(-1);
        }
    };

    if config.log.logo {
        logo(&logger_server);
    }

    if config.sentinels.is_some() {
        // TODO create channel to allow communicate between client connection and sentinel watcher
        if let Err(e) = watch_client(&config, logger_client) {
            error!(logger_server, "Error from listen client : {:?}", e);
        }

        if let Err(e) = watch_sentinel(&config, &logger_server) {
            error!(logger_server, "Error when running : {:?}", e);
        }
    } else {
        error!(logger_server, "No sentinels found in config file");
    }
}
