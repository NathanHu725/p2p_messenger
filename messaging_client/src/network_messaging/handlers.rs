use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::sync::{Arc, Mutex};

use super::utils::write_message;

type HandlerResult = Result<Result<String, String>, String>;
pub type CacheMap = Arc<Mutex<HashMap<String, String>>>;

const MDIR: &str = "./messages/";
pub const DELIMITER: &str = "&&";

/*
 * A special handler for two client-server messages
*/

pub fn handle_main_server_connection(
    mut stream: &TcpStream,
    recip: &str,
    user: &str,
) -> Option<Result<String, String>> {
    // Read the message into a buffer
    let mut buffer = [0; 2048];

    // Split the message into a status line and a body
    if let Ok(i) = stream.read(&mut buffer) {
        if i == 0 {
            println!("Main Server Shutdown");
            exit(0);
        }

        let as_string = std::str::from_utf8(&buffer[..i]).unwrap();

        // Handle based on the status code
        if let Some((code, message)) = as_string.split_once(" ") {
            let response: HandlerResult = match code {
                "ACK" => handle_acker(message, recip),
                "SEND" => handle_send(message, recip, user),
                "404" => handle_not_found(message),
                _ => handle_error(message),
            };

            return Some(Ok(match response {
                Ok(Ok(returner)) => returner,
                // Should never be reached
                Ok(Err(returner)) => returner,
                Err(returner) => returner,
            }));
        };
    }

    None
}

/*
 * A general handle connection method that decides which handle to use.
 * This is mainly used for the server.
*/

pub fn handle_connection(
    mut stream: &TcpStream,
    recip: &str,
    user: &str,
    cache: &mut CacheMap,
) -> Option<Result<String, String>> {
    // Read the message into a buffer
    let mut buffer = [0; 2048];

    // Split the message into a status line and a body
    if let Ok(i) = stream.read(&mut buffer) {
        if i == 0 {
            println!("Main Server Shutdown");
            exit(0);
        }

        let as_string = std::str::from_utf8(&buffer[..i]).unwrap();
        println!("{}", as_string);

        // Handle based on the status code
        if let Some((code, message)) = as_string.split_once(" ") {
            let response: HandlerResult = match code {
                "ACK" => handle_acker(message, recip),
                "INIT" => handle_init(message, cache),
                "SEND" => handle_send(message, recip, user),
                "CACHE" => handle_cache(message, cache),
                "404" => handle_not_found(message),
                _ => handle_error(message),
            };

            // Take action based on the result
            if let Err(reply) = response {
                stream.write_all(reply.as_bytes()).unwrap();
                stream.flush().unwrap();
            } else if let Ok(returner) = response {
                return Some(returner);
            }

            return None;
        };
    }

    None
}

/*
 * Handles an ack by writing the message locally (confirmed delivery)
*/

fn handle_acker(message: &str, recip: &str) -> HandlerResult {
    // Pull the original message out
    let (username, orig_message) = message.split_once(";").unwrap();

    // Construct a filename based on directory and username
    let file_name: String = MDIR.to_owned() + username + ".txt";

    // Write the original message to the appropriate file
    write_message(file_name, &("You;".to_owned() + orig_message));

    // Print to stdout if it matches the current recipt
    if username == recip {
        let formatted_t = &Utc::now().to_rfc2822()[..25];
        println!("{} You -> {}", formatted_t, orig_message);
    }

    // No response required
    Ok(Ok(String::from("")))
}

/*
 * Receive an ip retrieval message from the server
*/

pub fn handle_ip_retrieval(stream: &mut TcpStream) -> Option<String> {
    let mut buffer = [0; 2048];

    // Split the message into a status line and a body
    if let Ok(i) = stream.read(&mut buffer) {
        let as_string = std::str::from_utf8(&buffer[..i]).unwrap();

        // Split the code of the message
        if let Some((code, message)) = as_string.split_once(" ") {
            match code {
                "IP_RETRIEVAL" => return match message {
                    "" => None,
                    _ => Some(message.to_string()),
                },
                _ => return None,
            };
        };
    }

    None
}

/*
 * Receive an ack from a message sent from the main thread
*/

pub fn handle_ack(stream: &mut TcpStream, recip: &str) {
    let mut buffer = [0; 2048];

    // Split the message into a status line and a body
    if let Ok(i) = stream.read(&mut buffer) {
        let as_string = std::str::from_utf8(&buffer[..i]).unwrap();

        // Split the code of the message
        if let Some((code, message)) = as_string.split_once(" ") {
            match code {
                "ACK" => {
                    // Pull the original message out
                    let (username, orig_message) = message.split_once(";").unwrap();

                    // Construct a filename based on directory and username
                    let file_name: String = MDIR.to_owned() + username + ".txt";

                    // Write the original message to the appropriate file
                    write_message(file_name, &("You;".to_owned() + orig_message));

                    // Print to stdout if it matches the current recipt
                    if username == recip {
                        let formatted_t = &Utc::now().to_rfc2822()[..25];
                        println!("{} You -> {}", formatted_t, orig_message);
                    }
                }
                _ => println!("Invalid update message: {}", as_string),
            }
        }
    }
}

/*
 * Receive the cache update from the server
*/

pub fn handle_update(stream: &mut TcpStream, message_set: &mut HashSet<String>) {
    let mut buffer = [0; 2048];

    // Split the message into a status line and a body
    if let Ok(i) = stream.read(&mut buffer) {
        let as_string = std::str::from_utf8(&buffer[..i]).unwrap();

        // Split the code of the message
        if let Some((code, message)) = as_string.split_once(" ") {
            match code {
                "UPDATE" => {
                    println!("Received update: {}", message);
                    let mut messages = message.split(DELIMITER);
                    while let Some(message) = messages.next() {
                        message_set.insert(message.to_string());
                    }
                }
                _ => println!("Invalid update message: {}", as_string),
            }
        }
    }
}

/*
 * Write the sent message locally, then return an ack
*/

fn handle_send(message: &str, recip: &str, user: &str) -> HandlerResult {
    // Split sender and message
    let (sender, orig_message) = message.split_once(";").unwrap();

    // Construct a filename based on directory and username
    let file_name: String = MDIR.to_owned() + sender + ".txt";

    // Write the original message to the appropriate file
    write_message(file_name, message);

    // Print to stdout if it matches the current recipt
    if sender == recip {
        let formatted_t = &Utc::now().to_rfc2822()[..25];
        println!("{} {} -> {}", formatted_t, sender, orig_message);
    }

    Err("ACK ".to_owned() + user + ";" + orig_message)
}

/*
 * Return the list of buddies from the stream
*/

pub fn handle_buddies(stream: &mut TcpStream) -> Option<String> {
    let mut buffer = [0; 2048];

    // Split the message into a status line and a body
    if let Ok(i) = stream.read(&mut buffer) {
        let as_string = std::str::from_utf8(&buffer[..i]).unwrap();

        // Split the code of the message
        if let Some((code, message)) = as_string.split_once(" ") {
            match code {
                "BUDDIES" => return Some(message.to_string()),
                _ => return None,
            }
        }
    }

    None
}

/*
 * Handles a cache message for a potential buddy
*/

fn handle_cache(message: &str, cache: &mut CacheMap) -> HandlerResult {
    // Split the message from the recipient
    let (recip, cached_message) = message.split_once(";").unwrap();

    let mut cache = cache.lock().unwrap();

    // Get any existing cached messages
    let existing_cache = match cache.get(recip) {
        Some(existing_cache) => existing_cache.clone(),
        None => "".to_string(),
    };

    // Insert the new message appended to the existing messages
    cache.insert(
        recip.to_owned(),
        existing_cache + DELIMITER + cached_message,
    );
    Ok(Ok(String::from("")))
}

/*
 * Handles an init message from a buddy
*/

fn handle_init(message: &str, cache: &mut CacheMap) -> HandlerResult {
    let cached_messages = cache.lock().unwrap().insert(message.to_owned(), "".to_owned());

    // If there were cached messages, send them to the buddy
    if let Some(cached_messages) = cached_messages {
        Err("UPDATE ".to_owned() + &cached_messages)
    } else {
        Err("UPDATE ".to_owned() + DELIMITER)
    }
}

/*
 * Handle error - should not be creached
*/

fn handle_error(message: &str) -> HandlerResult {
    // We don't know how to handle this request, so send that to main thread
    Ok(Err("404 ".to_owned() + message))
}

/*
 * Handle 404 errors - user or command not found/supported
*/

fn handle_not_found(message: &str) -> HandlerResult {
    println!("{}", message);
    Ok(Err("404 ".to_owned() + message))
}
