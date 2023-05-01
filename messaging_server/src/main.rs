use std::collections::HashMap;
use local_ip_address::local_ip;
use mio::net::TcpListener;
use mio::{Events, Poll, Token, Interest};
use std::io::{self, Read};
use std::process;
use handlers::{CacheMap, ConnMap, SockMap, handle_send, handle_ack, handle_error, handle_init, handle_ip_retrieval};

const PORT: u16 = 8013;
const LISTENER: Token = Token(0);

fn listener_poll(listener: &mut TcpListener, poll: &Poll, sockets: &mut SockMap, socket_index: &mut usize) {
    loop {
        match listener.accept() {
            Ok((mut socket, _)) => {
                // Get the token for the socket
                let token = Token(*socket_index);
                *socket_index += 1;

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

fn token_poll(poll: &Poll, token: &Token, sockets: &mut SockMap, buf: &mut [u8], connections: &mut ConnMap, cache: &mut CacheMap) {
    loop {
        if let Some(stream) = sockets.get_mut(&token) {
            println!("This is the reading");
            match stream.read(buf) {
                Ok(0) => {
                    // Socket is closed, remove it from the map
                    sockets.remove(&token);
                    break;
                }
                // Data is not actually sent in this example
                Ok(i) => {
                    let (code, message) = std::str::from_utf8(&buf[..i]).unwrap().split_once(" ").unwrap();
                    println!("This is the message: {}:{}", code, message);
                    // Handle based on the status code
                    let t_val = match code {
                        "ACK" => handle_ack(message, cache),
                        "SEND" => handle_send(token, sockets, message, connections, cache),
                        "INIT" => handle_init(token, sockets, message, connections, cache),
                        "IP_FETCH" => handle_ip_retrieval(token, sockets, message, connections),
                        "SHUTDOWN" => process::exit(0),
                        _ => handle_error(message),
                    };
    
                    if let Some(t) = t_val {
                        let mut socket = sockets.remove(token).unwrap();
                        poll.registry().reregister(&mut socket, Token(t), 
                            Interest::READABLE | Interest::WRITABLE).unwrap();
                        sockets.remove(&token);
                        sockets.insert(Token(t), socket);
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Socket is not ready anymore, stop reading
                    break;
                }
                e => println!("err={:?}", e), // Unexpected error
            }
        } else {
            break;
        }
    }
}

fn run_server(mut conn: ConnMap, mut cache: CacheMap) {
    // Create poll and appropriate objects
    let mut poll = Poll::new().unwrap();
    let mut sockets: SockMap = HashMap::new();
    let mut events = Events::with_capacity(1024);
    let mut socket_index = 1;

    // Create listener and buffer
    let mut listener = TcpListener::bind(
        format!("{:?}:{}", local_ip().unwrap(), 
            PORT).parse().unwrap()
    ).unwrap();
    poll.registry().register(&mut listener, LISTENER, Interest::READABLE).unwrap();

    loop {
        // Wait for events
        poll.poll(&mut events, None).unwrap();

        // Iterate through events
        for event in &events {
            let mut buf = [0; 1024];
            match event.token() {
                LISTENER => {
                    listener_poll(&mut listener, &poll, &mut sockets, &mut socket_index);
                }
                token => {
                    token_poll(&poll, &token, &mut sockets, &mut buf, &mut conn, &mut cache);
                }
            }
        }
    }
}

fn main() {
    let active_connections: ConnMap = HashMap::new();
    let cached_messages: CacheMap = HashMap::new();
    run_server(active_connections, cached_messages);

    println!("Hello, world!");
}