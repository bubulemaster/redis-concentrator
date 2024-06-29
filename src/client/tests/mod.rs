use crate::client::manage_new_client_message;
use crate::config::{Config, ConfigLog};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

#[test]
fn test_manage_new_client_message_ok() {
    let (tx_new_client, rx_new_client): (
        Sender<(TcpStream, SocketAddr)>,
        Receiver<(TcpStream, SocketAddr)>,
    ) = mpsc::channel();

    let mut client_map = HashMap::new();
    let mut redis_master_addr;

    // Create fake Redis server
    let listener_redis_server = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => panic!("Error to create test server: {:?}", e),
    };

    redis_master_addr = format!(
        "127.0.0.1:{}",
        listener_redis_server.local_addr().unwrap().port()
    );

    let client = match TcpStream::connect(&redis_master_addr) {
        Ok(l) => l,
        Err(e) => panic!("Error to create test client: {:?}", e),
    };

    let socket = client.local_addr().unwrap();

    tx_new_client.send((client, socket)).unwrap();

    manage_new_client_message(
        &rx_new_client,
        &mut client_map,
        &mut redis_master_addr,
    );

    assert_eq!(client_map.len(), 1);
}
