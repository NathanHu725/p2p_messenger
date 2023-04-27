use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use local_ip_address::local_ip;
use mio::net::TcpListener;
use mio::{Events, Poll, Token, Interest};
use std::io::{self, Read};
use std::thread;
use handlers::{CacheMap, ConnMap, handle_ack, handle_connection, handle_error, handle_init, handle_ip_retrieval};

const PORT: u16 = 8013;
const LISTENER: Token = Token(0);

fn setup_server(conn: ConnMap, cache: CacheMap) {
    // Create poll and appropriate objects
    let mut poll = Poll::new().unwrap();
    let mut sockets = HashMap::new();
    let mut events = Events::with_capacity(1024);
    let mut socket_index = 1;

    // Create listener and buffer
    let listener = TcpListener::bind(
        format!("{:?}:{}", local_ip().unwrap(), 
            PORT).parse().unwrap()
    ).unwrap();
    let mut buf = [0; 1024];

    loop {
        // Wait for events
        poll.poll(&mut events, None).unwrap();

        // Iterate through events
        for event in &events {
            match event.token() {
                LISTENER => {
                    loop {
                        match listener.accept() {
                            Ok((mut socket, _)) => {
                                // Get the token for the socket
                                let token = Token(socket_index);
                                socket_index += 1;
    
                                // Register the new socket w/ poll
                                poll.registry().register(&mut socket, token, 
                                    Interest::READABLE | Interest::WRITABLE).unwrap();
    
                                // Store the socket
                                sockets.insert(token, socket);
                            }
                            // Socket is not ready anymore, stop accepting
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                break;
                            }
                            // Unexpected error
                            e => panic!("err={:?}", e), 
                        }
                    }
                }
                token => {
                    loop {
                        match sockets.get_mut(&token).unwrap().read(&mut buf) {
                            Ok(0) => {
                                // Socket is closed, remove it from the map
                                sockets.remove(&token);
                                break;
                            }
                            // Data is not actually sent in this example
                            Ok(message) => {
                                let (code, message) = match as_string.split_once(" ").unwrap();
                                
                                // Handle based on the status code
                                match code {
                                    "ACK" => handle_ack(message, cache),
                                    "SEND" => handle_send(stream, message, connections, cache),
                                    "INIT" => handle_init(stream, message, connections, cache),
                                    "IP_RETRIEVAL" => handle_ip_retrieval(stream, message, connections),
                                    _ => handle_error(message),
                                };
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                // Socket is not ready anymore, stop reading
                                break;
                            }
                            e => panic!("err={:?}", e), // Unexpected error
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let active_connections = Arc::new(Mutex::new(HashMap::new()));
    let cached_messages = Arc::new(Mutex::new(HashMap::new()));
    setup_server(active_connections.clone(), cached_messages.clone());

    println!("Hello, world!");
}