use std::io::Write;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::collections::HashSet;

use super::handlers::{handle_buddies, handle_update, DELIMITER};
use super::utils::write_message;

const SERVER: &str = "limia.cs.williams.edu:8013";

/*
 * Creates the tcp connection to the main server and sends an init
 * message based on the entered username
*/

pub fn initialize(username: &str, ip_addr: &str, port: u16) -> Option<TcpStream> {
    let stream = init_stream(SERVER);

    match stream {
        Ok(mut server) => {
            let message = [
                "INIT ".as_bytes(),
                username.as_bytes(),
                DELIMITER.as_bytes(),
                ip_addr.as_bytes(),
                ":".as_bytes(),
                port.to_string().as_bytes(),
            ]
            .concat();

            // Send the buddies message and what to do with the buddies
            send_to_buddies(&message, &mut server, &username, | buddy_list | {
                let mut buddies = buddy_list.split(DELIMITER);
                let mut new_messages = HashSet::new();

                while let Some(buddy) = buddies.next() {
                    if let Ok(mut stream) = init_stream(&buddy) {
                        _ = send_message(
                            &["INIT ".as_bytes(), username.as_bytes()].concat(),
                            &mut stream,
                        );

                        handle_update(&mut stream, &mut new_messages);
                    }
                }

                // Iterate through the messages, write them locally
                for message in new_messages {
                    let (recipient, message) = message.split_once(";").unwrap();
                    write_message(recipient.to_string(), message);
                };

                "Updated Messages".to_string()
            });

            println!("Welcome to Jaelegram");

            // Set up the server and input stream to be non_blocking
            _ = server.set_nonblocking(true);
            Some(server)
        }
        Err(_) => None,
    }
}

/*
 * A helper method to make connecting easier
*/

pub fn init_stream(addr: &str) -> Result<TcpStream, std::io::Error> {
    if addr != "" {
        TcpStream::connect(addr.to_socket_addrs().unwrap().next().unwrap())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid Address",
        ))
    }
}

/*
 * Creates ip_fetch method and sends it
*/

pub fn ip_fetch(recipient: &str, server: &TcpStream) -> Option<String> {
    let message = "IP_FETCH ".to_owned() + recipient;
    send_message(message.as_bytes(), server)
}

/*
 * Sends a message to a stream
*/

pub fn send_message(message: &[u8], mut server: &TcpStream) -> Option<String> {
    _ = server.write(message);
    _ = server.flush();
    Some(String::from("Sent"))
}

/*
 * Send the message to retrieve buddies. Once the list is received, the
 * buddies are split and the message is sent to all of them
*/

pub fn send_backups(
    recip_copy: &str,
    username: &str,
    message: &str,
    server: &mut TcpStream,
) -> Option<String> {
    // Create the buddies message
    let buddy_mes = ["BUDDIES ".as_bytes(), recip_copy.as_bytes()].concat();

    // Send the buddies message and what to do with the buddies
    send_to_buddies(&buddy_mes, server, &username, | buddy_list | {
        let mut buddies = buddy_list.split(DELIMITER);
        let mut counter = 0;

        while let Some(buddy) = buddies.next() {
            if let Ok(mut stream) = init_stream(&buddy) {
                _ = send_message(
                    &["CACHE ".as_bytes(), recip_copy.as_bytes(), ";".as_bytes(), username.as_bytes(), ";".as_bytes(), message.as_bytes()].concat(),
                    &mut stream,
                );
                counter += 1;
            }
        }

        if counter == 0 {
            "No buddies online".to_string()
        } else {
            "Sent".to_string()
        }
    })
}

/*
 * Handle all a buddies request given a closure
*/

fn send_to_buddies<F: Fn(String) -> String>(buddies_message: &[u8], server: &mut TcpStream, username: &str, f: F) -> Option<String> {
    _ = send_message(buddies_message, &server);
    match handle_buddies(server, username) {
        Some(buddy_list) => {
            Some(f(buddy_list))
        }
        None => None,
    }
}
