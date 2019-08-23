#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate log;
#[macro_use]
extern crate slog;

mod config;
mod lib;
mod logging;
mod sentinel;

use std::env;
use std::net::TcpStream;

use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::RedisConnector;
use crate::config::get_config;
use crate::sentinel::watch_sentinel;
use crate::logging::create_log;

const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

fn help() {
    println!("redis-concentrator {}", VERSION.unwrap_or("unknown"));
    println!();
    println!("Usage: redis-concentrator config-file");
    println!();
}

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
    let logger = match create_log(&config) {
        Some(l) => l,
        None => {
            eprintln!("Error: cannot create log!");
            std::process::exit(-1);
        }
    };

    if config.log.header {
        // Thanks to http://patorjk.com/software/taag/#p=display&f=Doom&t=Redis%20Concentrator
        info!(logger, r"______         _ _       _____                            _             _             ");
        info!(logger, r"| ___ \       | (_)     /  __ \                          | |           | |            ");
        info!(logger, r"| |_/ /___  __| |_ ___  | /  \/ ___  _ __   ___ ___ _ __ | |_ _ __ __ _| |_ ___  _ __ ");
        info!(logger, r"|    // _ \/ _` | / __| | |    / _ \| '_ \ / __/ _ \ '_ \| __| '__/ _` | __/ _ \| '__|");
        info!(logger, r"| |\ \  __/ (_| | \__ \ | \__/\ (_) | | | | (_|  __/ | | | |_| | | (_| | || (_) | |   ");
        info!(logger, r"\_| \_\___|\__,_|_|___/  \____/\___/|_| |_|\___\___|_| |_|\__|_|  \__,_|\__\___/|_|   ");
        info!(logger, "");
    }

    if config.sentinels.is_some() {
        watch_sentinel(&config);
    } else {
        error!(logger, "No sentinels found in config file");
    }
/*

    println!("SUBSCRIBE");

    match redis_connector.subscribe("+switch-master") {
        Ok(mut s) => {
            loop {
                let a = s.pool();
                println!("Pool result: {:?}", a);
            }
        },
        Err(e) => println!("Error: {:?}", e)
    };
*/
}
