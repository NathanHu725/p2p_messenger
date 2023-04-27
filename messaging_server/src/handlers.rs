use mio::net::TcpStream;
use mio::Token;
use std::io::Write;
use std::collections::HashMap;

pub type CacheMap = HashMap<String, Vec<String>>;
pub type ConnMap = HashMap<String, Token>;
pub type SockMap = HashMap<Token, TcpStream>;

// Inspired by rust handbook
// pub fn handle_connection(username: &str, mut stream: &TcpStream, connections: &ConnMap, cache: &CacheMap) {
//     // Read the message into a buffer
//     let mut buffer = [0; 1024];
//     stream.read(&mut buffer).unwrap();

//     // Split the message into a status line and a body
//     let as_string = std::str::from_utf8(&buffer).unwrap();
//     let (code, message) = match as_string.split_once(" ") {
//         Some((a, b)) => (a, b),
//         None => {
//             stream.shutdown(Shutdown::Both);
//             connections.lock().unwrap().remove(username);
//             ("ERROR", "")
//         },
//     };

//     // Handle based on the status code
//     match code {
//         "ACK" => handle_ack(message, cache),
//         "SEND" => handle_send(stream, message, connections, cache),
//         "INIT" => handle_init(stream, message, connections, cache),
//         "IP_RETRIEVAL" => handle_ip_retrieval(stream, message, connections),
//         _ => handle_error(message),
//     };
// }

pub fn handle_init(token: &Token, sockets: &mut SockMap, 
                message: &str, 
                connections: &mut ConnMap, 
                cache: &CacheMap) {
    println!("received init {}", message);
    // Split the message into tokens
    let (username, _ip) = message.split_once(";").unwrap();

    let mut default = Vec::<String>::new();
    let to_write = match cache.get(username) {
        Some(v) => v,
        None => &mut default,
    };

    let message = format!("UPDATE {}", to_write.join("\n"));
    let stream = sockets.get_mut(&token).unwrap();

    stream.write_all(message.as_bytes()).unwrap();
    stream.flush().unwrap();
    connections.insert(username.to_string(), *token);
}

pub fn handle_send(token: &Token, sockets: &mut SockMap, 
    message: &str, 
    connections: &mut ConnMap, 
    cache: &mut CacheMap) {
    println!("received send {}", message);
    // Pull the sender out
    let (sender, orig_message) = message.split_once(";").unwrap();
    let orig_message = orig_message.to_string();

    if let Some(t) = connections.get(sender) {
        let stream = sockets.get_mut(&t).unwrap();
        stream.write_all(message.as_bytes()).unwrap();
        stream.flush().unwrap();
        if let Some(v) = cache.get_mut(sender) {
            v.push(orig_message);
        } else {
            cache.insert(sender.to_string(), vec![orig_message]);
        }
        println!("Sent {}", message);
    } else {
        let message = format!("404 {} not found", sender);
        let stream = sockets.get_mut(&token).unwrap();
        stream.write_all(message.as_bytes()).unwrap();
        println!("Sent {}", message);
        stream.flush().unwrap();
    }
}

pub fn handle_ack(message: &str, 
    cache: &mut CacheMap) {
        println!("received ack {}", message);
    // Remove the message from the cache one there is a receipt
    let (username, orig_message) = message.split_once(";").unwrap();

    let mut default = Vec::<String>::new();
    let user_cache: &mut Vec<String> = match cache.get_mut(username) {
        Some(v) => v,
        None => &mut default,
    };
    let index = user_cache.iter().position(|x| *x == orig_message).unwrap();
    user_cache.remove(index);
}

pub fn handle_ip_retrieval(token: &Token, sockets: &mut SockMap, username: &str, connections: &ConnMap) {
    let message = match connections.get(username) {
        Some(t) => String::from("IP_RETRIEVAL ") + &sockets.get_mut(&t).unwrap().peer_addr().unwrap().to_string(),
        None => String::from("404 not found"),
    };

    let stream = sockets.get_mut(&token).unwrap();
    stream.write_all(message.as_bytes()).unwrap();
    stream.flush().unwrap();
}

pub fn handle_error(message: &str) {
    println!("received error {}", message);
}