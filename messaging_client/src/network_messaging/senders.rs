use std::net::TcpStream;
use std::io::Write;
use std::net::ToSocketAddrs;

use super::handlers::{handle_connection, DELIMITER};

const SERVER: &str = "limia.cs.williams.edu:8013";

pub fn initialize(username: &str, ip_addr: &str, port: u16) -> Option<TcpStream>{
    let stream = init_stream(SERVER);

    match stream {
        Ok(mut server) => {
            let message = ["INIT ".as_bytes(), 
                        username.as_bytes(),
                        ";".as_bytes(),
                        ip_addr.as_bytes(),
                        ":".as_bytes(),
                        port.to_string().as_bytes()].concat();
            _ = server.write(&message);
            _ = server.flush();

            handle_connection(&server, "", username);
            println!("Welcome to Jaelegram");

            // Set up the server and input stream to be non_blocking
            _ = server.set_nonblocking(true);
            Some(server)
        },
        Err(_) => None,
    }
}

pub fn init_stream(addr: &str) -> Result<TcpStream, std::io::Error> {
    TcpStream::connect(addr.to_socket_addrs().unwrap().next().unwrap())
}

pub fn ip_fetch(recipient: &str, mut server: &TcpStream) -> Option<String> {
    let message = ["IP_FETCH ".as_bytes(), recipient.as_bytes()].concat();
    _ = server.write(&message);
    _ = server.flush();
    Some(String::from("Sent"))
}

pub fn send_message(message: String, mut server: &TcpStream) -> Option<String> {
    _ = server.write(message.trim().as_bytes());
    _ = server.flush();
    Some(String::from("Sent"))
}

pub fn send_backups(recip_copy: &str, username: &str, message: &str, server: &TcpStream) -> Option<String> {
    let buddy_mes = "BUDDIES ".to_owned() + recip_copy;
    _ = send_message(buddy_mes, &server);

    match handle_connection(&server, recip_copy, username) {
        Some(Ok(buddy_list)) => {
            // Given a buddies response, we want to iterate through the buddies and send them the messages to cache
            let mut buddies = buddy_list.split(DELIMITER);
            let mut counter = 0;

            while let Some(buddy) = buddies.next() {
                if let Ok(mut stream) = init_stream(&buddy) {
                    _ = send_message(message.to_string(), &mut stream);
                    counter += 1;
                }
            }

            if counter == 0 {
                None
            } else {
                Some(String::from("Sent to Buddies"))
            }
        },
        Some(Err(_)) => None,
        None => None,
    }
}