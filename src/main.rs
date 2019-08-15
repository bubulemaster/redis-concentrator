#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::net::TcpStream;
//use std::io::ErrorKind;

mod config;
mod lib;

use std::env;

use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::RedisConnector;

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

    let tcp_stream = TcpStream::connect("127.0.0.1:26000").unwrap();

    let timeout = std::time::Duration::from_millis(1000);
    tcp_stream.set_read_timeout(Some(timeout));
    tcp_stream.set_nonblocking(false);

    let mut stream = NetworkStream::new(tcp_stream);
    let mut redis_connector = RedisConnector::new(&mut stream);

    /*println!("PING");

    match redis_connector.ping() {
        Ok(s) => println!("read: {:?}", s),
        Err(e) => println!("Error: {:?}", e)
    };*/

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

    /*match redis_connector.get_string("a") {
        Ok(s) => println!("read: {:?}", s),
        Err(e) => println!("Error: {:?}", e)
    };*/
}
