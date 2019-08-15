use std::net::TcpStream;
//use std::io::ErrorKind;

mod lib;

use crate::lib::redis::stream::network::NetworkStream;
use crate::lib::redis::RedisConnector;
/*
fn ping(stream: &mut NetworkStream) ->  std::io::Result<String> {
    let cmd = "PING\r\n".as_bytes();

    stream.write(cmd)?;

    match stream.get_until("\r\n".as_bytes()) {
        Ok(buf) =>
            match std::str::from_utf8(&buf) {
                Ok(v) => Ok(String::from(v)), // TODO check response
                Err(e) => Err(
                    std::io::Error::new(
                        ErrorKind::InvalidData,
                        format!("Invalid UTF-8 sequence: {}", e)))
            },
        Err(e) => Err(e)
    }
}

fn subscribe(stream: &mut NetworkStream) ->  std::io::Result<()> {
    let cmd = "SUBSCRIBE +switch-master\r\n".as_bytes();

    stream.write(cmd)?;

    loop {
        match stream.get_until("\r\n".as_bytes()){
            Ok(buf) => {
                if buf.len() > 0 {
                    println!("Receive: {:?}", buf);

                    match std::str::from_utf8(&buf) {
                        Ok(v) => println!("{}", String::from(v)), // TODO check response
                        Err(e) => return Err(
                            std::io::Error::new(
                                ErrorKind::InvalidData,
                                format!("Invalid UTF-8 sequence: {}", e)))
                    }
                }
            },
            Err(e) => {
                println!("******* Read closed");
                return Err(e)
            }
        }
    }
}
*/
fn main() {
    println!("Hello, world!");

    let mut tcp_stream = TcpStream::connect("127.0.0.1:26000").unwrap();

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
