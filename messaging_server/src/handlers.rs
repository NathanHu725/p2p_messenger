use mio::net::TcpStream;
use mio::Token;
use std::io::Write;
use std::collections::HashMap;

pub type CacheMap = HashMap<String, Vec<String>>;
pub type ConnMap = HashMap<String, (String, Token)>;
pub type SockMap = HashMap<Token, TcpStream>;

pub fn handle_init(token: &Token, sockets: &mut SockMap, 
                message: &str, 
                connections: &mut ConnMap, 
                cache: &mut CacheMap) -> Option<usize> {
    // Split the message into tokens
    let (username, ip) = message.split_once(";").unwrap();

    // Get the cache info for an update
    let mut default = Vec::<String>::new();
    let to_write = match cache.get(username) {
        Some(v) => v,
        None => &mut default,
    };

    // Send the update message
    let message = format!("UPDATE {}", to_write.join("&&"));
    let stream = sockets.get_mut(&token).unwrap();
    stream.write_all(message.as_bytes()).unwrap();
    stream.flush().unwrap();

    
    // Insert a cleared cache
    cache.insert(username.to_string(), default);

    // See if this user exists
    match connections.get(username) {
        Some((_, Token(t))) => {
            // If they do, reregister with existing token #
            return Some(*t);
        },
        None => {
            // If they do not, register them in connections arr
            connections.insert(username.to_string(), (ip.to_string(), *token));
        },
    };

    None
}

pub fn handle_send(token: &Token, sockets: &mut SockMap, 
    message: &str, 
    connections: &mut ConnMap, 
    cache: &mut CacheMap) -> Option<usize> {
    // Pull the sender and receiver out
    let (receiver, orig_message) = message.split_once(";").unwrap();
    let (_, encrypted_message) = orig_message.split_once(";").unwrap();

    if let Some((_, t)) = connections.get(receiver) {
        if let Some(stream) = sockets.get_mut(&t) {
            stream.write_all(&["SEND ".as_bytes(), orig_message.as_bytes()].concat()).unwrap();
            stream.flush().unwrap();
            println!("Sent {}", message);
        }

        let message = format!("ACK {};{}", receiver, encrypted_message);
        let orig_message = orig_message.to_string();
        if let Some(v) = cache.get_mut(receiver) {
            v.push(orig_message);
        } else {
            cache.insert(receiver.to_string(), vec![orig_message]);
        }

        let stream = sockets.get_mut(&token).unwrap();
        stream.write_all(message.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {
        let message = format!("404 {} not found", receiver);
        let stream = sockets.get_mut(&token).unwrap();
        stream.write_all(message.as_bytes()).unwrap();
        println!("Sent {}", message);
        stream.flush().unwrap();
    }
    None
}

pub fn handle_ack(message: &str, 
    cache: &mut CacheMap) -> Option<usize> {
    // Remove the message from the cache one there is a receipt
    let (username, orig_message) = message.split_once(";").unwrap();

    let mut default = Vec::<String>::new();
    let user_cache: &mut Vec<String> = match cache.get_mut(username) {
        Some(v) => v,
        None => &mut default,
    };
    if let Some(index) = user_cache.iter().position(|x| *x == orig_message) {
        user_cache.remove(index);
    }
    None
}

pub fn handle_ip_retrieval(token: &Token, sockets: &mut SockMap, username: &str, connections: &ConnMap) -> Option<usize> {
    let message = match connections.get(username) {
        Some((addr, _)) => String::from("IP_RETRIEVAL ") + addr,
        None => String::from("404 not found"),
    };

    let stream = sockets.get_mut(&token).unwrap();
    stream.write_all(message.as_bytes()).unwrap();
    stream.flush().unwrap();
    None
}

pub fn handle_error(message: &str) -> Option<usize> {
    println!("received error {}", message);
    None
}