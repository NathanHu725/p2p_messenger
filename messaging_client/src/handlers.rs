mod utils;
use utils::write_message;
use std::net::TcpStream;
use std::io::{Write, Read};
use std::sync::mpsc::Sender;

const MDIR: &str = "./messages/";

// Inspired by rust handbook
pub fn handle_connection(mut stream: &TcpStream) -> Option<String> {
    // Read the message into a buffer
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Split the message into a status line and a body
    let as_string = std::str::from_utf8(&buffer).unwrap();
    let (code, message) = as_string.split_once(" ").unwrap();

    // Handle based on the status code
    let response: Result<String, String> = match code {
        "ACK" => handle_ack(message),
        "SEND" => handle_send(message),
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

    None
}

// Handle the ack, this means writing the message locally
fn handle_ack(message: &str) -> Result<String, String> {
    // Pull the original message out
    let (username, orig_message) = message.split_once(";").unwrap();

    // Construct a filename based on directory and username
    let file_name: String = MDIR.to_owned() + username + ".txt";

    // Write the original message to the appropriate file
    write_message(file_name, &("You;".to_owned() + orig_message));

    // No response required
    Ok(String::from(""))
}

fn handle_update(message: &str) -> Result<String, String> {
    // Split the messages
    let mut message_tokens = message.split("\n");

    while let Some(in_message) = message_tokens.next() {
        if let Some((username, m)) = in_message.split_once(";") {
            // Construct a filename based on directory and username
            let file_name: String = MDIR.to_owned() + username + ".txt";

            // Write the original message to the appropriate file
            write_message(file_name, m);
        }
    }

    Ok(String::from(""))
}

fn handle_send(message: &str) -> Result<String, String> {
    // Pull the sender out
    let (sender, orig_message) = message.split_once(";").unwrap();

    // Construct a filename based on directory and username
    let file_name: String = MDIR.to_owned() + sender + ".txt";

    // Write the original message to the appropriate file
    write_message(file_name, message);

    Err("ACK ".to_owned() + message)
}

fn handle_ip_retrieval(message: &str) -> Result<String, String> {
    // Forward either the ip address of the person we requested or error to the main server
    Ok(String::from(message))
}

fn handle_error(message: &str) -> Result<String, String> {
    // We don't know how to handle this request, so send that to main thread
    Err("404 " .to_owned()+ message)
}

fn handle_not_found(message: &str) -> Result<String, String> {
    println!("{}", message);
    Ok("404 ".to_owned() + message)
}