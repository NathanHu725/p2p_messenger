use std::net::{TcpStream, Shutdown};
use std::io::{Write, Read};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::boxed::Box;

const MDIR: &str = "./messages/";

// Inspired by rust handbook
pub fn handle_connection(username: &str, mut stream: TcpStream, connections: Arc<Mutex<HashMap<String, TcpStream>>>, cache: Arc<Mutex<HashMap<String, Vec<String>>>>) {
    // Read the message into a buffer
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Split the message into a status line and a body
    let as_string = std::str::from_utf8(&buffer).unwrap();
    let (code, message) = match as_string.split_once(" ") {
        Some((a, b)) => (a, b),
        None => {
            stream.shutdown(Shutdown::Both);
            connections.lock().unwrap().remove(username);
            ("ERROR", "")
        },
    };

    // Handle based on the status code
    match code {
        "ACK" => handle_ack(message, cache),
        "SEND" => handle_send(stream, message, connections, cache),
        "INIT" => handle_init(stream, message, connections, cache),
        "IP_RETRIEVAL" => handle_ip_retrieval(stream, message, connections),
        _ => handle_error(message),
    };
}

fn handle_init(mut stream: TcpStream, 
                message: &str, 
                connections: Arc<Mutex<HashMap<String, TcpStream>>>, 
                cache: Arc<Mutex<HashMap<String, Vec<String>>>>) {
    println!("received init {}", message);
    // Split the message into tokens
    let (u, i) = message.split_once(";").unwrap();
    let (username, _ip) = (String::from(u), String::from(i));

    let default = Vec::<String>::new();
    let cache_access = cache.lock().unwrap();
    let to_write = match cache_access.get(&username) {
        Some(v) => v,
        None => &default,
    };

    let message = format!("UPDATE {}", to_write.join("\n"));
    drop(cache_access);

    stream.write_all(message.as_bytes()).unwrap();
    stream.flush().unwrap();
    connections.lock().unwrap().insert(username.clone(), stream);
}

fn handle_send(mut stream: TcpStream, 
    message: &str, 
    connections: Arc<Mutex<HashMap<String, TcpStream>>>, 
    cache: Arc<Mutex<HashMap<String, Vec<String>>>>) {
    println!("received send {}", message);
    // Pull the sender out
    let (s, o) = message.split_once(";").unwrap();
    let (sender, orig_message) = (String::from(s), String::from(o));

    if let Some(conn) = connections.lock().unwrap().get_mut(&sender) {
        conn.write_all(message.as_bytes()).unwrap();
        conn.flush().unwrap();
        if let Some(v) = cache.lock().unwrap().get_mut(&sender) {
            v.push(orig_message.to_owned());
            drop(v)
        } else {
            cache.lock().unwrap().insert(sender, vec![orig_message.to_owned()]);
        }
        println!("Sent {}", message);
    } else {
        let message = format!("404 {} not found", sender);
        stream.write_all(message.as_bytes()).unwrap();
        println!("Sent {}", message);
        stream.flush().unwrap();
    }
}

fn handle_ack(message: &str, 
    cache: Arc<Mutex<HashMap<String, Vec<String>>>>) {
        println!("received ack {}", message);
    // Remove the message from the cache one there is a receipt
    let (u, o) = message.split_once(";").unwrap();
    let (username, orig_message) = (String::from(u), String::from(o));

    let mut binding = cache.lock().unwrap();
    let user_cache = binding.get_mut(&username).unwrap();
    let index = user_cache.iter().position(|x| *x == orig_message).unwrap();
    user_cache.remove(index);
}

fn handle_ip_retrieval(mut stream: TcpStream, username: &str, connections: Arc<Mutex<HashMap<String, TcpStream>>>) {
    let message = match connections.lock().unwrap().get(username) {
        Some(conn) => String::from("IP_RETRIEVAL ") + &conn.peer_addr().unwrap().to_string(),
        None => String::from("404 not found"),
    };

    stream.write_all(message.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn handle_error(message: &str) {
    
}