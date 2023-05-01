mod utils;
use utils::write_message;
use chrono::Utc;
use std::process::exit;
use std::net::TcpStream;
use std::io::{Write, Read};

const MDIR: &str = "./messages/";

// Inspired by rust handbook
pub fn handle_connection(mut stream: &TcpStream, recip: &str, user: &str) -> Option<Result<String, String>> {
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
            let response: Result<Result<String, String>, String> = match code {
                "ACK" => handle_ack(message, recip),
                "SEND" => handle_send(message, recip, user),
                "UPDATE" => handle_update(message),
                "IP_RETRIEVAL" => handle_ip_retrieval(message),
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

// Handle the ack, this means writing the message locally
fn handle_ack(message: &str, recip: &str) -> Result<Result<String, String>, String> {
    // Pull the original message out
    let (username, orig_message) = message.split_once(";").unwrap();

    // Construct a filename based on directory and username
    let file_name: String = MDIR.to_owned() + username + ".txt";

    // Write the original message to the appropriate file
    write_message(file_name, &("You;".to_owned() + orig_message));

    // Print to stdout if it matches the current recipt
    if username == recip {
        let formatted_t = &Utc::now(). to_rfc2822()[..25];
        println!("{} You -> {}", formatted_t, orig_message);
    }

    // No response required
    Ok(Ok(String::from("")))
}

fn handle_update(message: &str) -> Result<Result<String, String>, String> {
    // Split the messages
    let mut message_tokens = message.split("&&");

    while let Some(in_message) = message_tokens.next() {
        if let Some((username, _)) = in_message.split_once(";") {
            // Construct a filename based on directory and username
            let file_name: String = MDIR.to_owned() + username + ".txt";

            // Write the original message to the appropriate file
            write_message(file_name, in_message);
        }
    }

    Ok(Ok(String::from("")))
}

fn handle_send(message: &str, recip: &str, user: &str) -> Result<Result<String, String>, String> {
    // Split sender and message
    let (sender, orig_message) = message.split_once(";").unwrap();

    // Construct a filename based on directory and username
    let file_name: String = MDIR.to_owned() + sender + ".txt";

    // Write the original message to the appropriate file
    write_message(file_name, message);

    // Print to stdout if it matches the current recipt
    if sender == recip {
        let formatted_t = &Utc::now(). to_rfc2822()[..25];
        println!("{} {} -> {}", formatted_t, sender, orig_message);
    }

    Err("ACK ".to_owned() + user + ";" + orig_message)
}

fn handle_ip_retrieval(message: &str) -> Result<Result<String, String>, String> {
    // Forward either the ip address of the person we requested or error to the main server
    Ok(Ok(String::from(message)))
}

fn handle_error(message: &str) -> Result<Result<String, String>, String> {
    // We don't know how to handle this request, so send that to main thread
    Ok(Err("404 " .to_owned()+ message))
}

fn handle_not_found(message: &str) -> Result<Result<String, String>, String> {
    println!("{}", message);
    Ok(Err("404 ".to_owned() + message))
}