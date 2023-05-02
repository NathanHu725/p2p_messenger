use mio::net::TcpStream;
use mio::Token;
use std::io::Write;
use std::convert::From;
use std::collections::HashMap;

mod utils;
use utils::{User, calculate_hash};

pub type CacheMap = HashMap<String, Vec<String>>;
pub type ConnMap = HashMap<String, User>;
pub type SockMap = HashMap<Token, TcpStream>;
pub type UserList = Vec<String>;

// Hyperparameter defining group size
const GROUP_SIZE: u32 = 10;

pub fn handle_init(
    token: &Token, 
    sockets: &mut SockMap, 
    message: &str, 
    connections: &mut ConnMap, 
    cache: &mut CacheMap,
    user_list: &UserList
) -> Option<usize> {
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
    write_m(sockets.get_mut(&token).unwrap(), message);

    
    // Insert a cleared cache
    cache.insert(username.to_string(), default);

    // See if this user exists
    match connections.get_mut(username) {
        Some(user) => {
            // If they do, update total and reregister with existing token #
            user.total_users = user_list.len() as u32;
            return Some(usize::from(user.token));
        },
        None => {
            // If they do not, register them in connections arr
            let new_user = User {
                token: *token,
                ip_addr: ip.to_string(),
                total_users: user_list.len() as u32,
            };
            connections.insert(username.to_string(), new_user);
        },
    };

    None
}

/*
 * Creates a string of ip_addrs that represent a users group
*/

pub fn handle_buddies(
    token: &Token, 
    sockets: &mut SockMap, 
    username: &str,
    connections: &ConnMap,
    user_list: &UserList
) -> Option<usize> {
    // Try to get the user from the connections table
    if let Some(user) = connections.get(username) {
        // Function to get evenly distributed, but also changing buddies
        let t = user.total_users;
        let seed: u32 = calculate_hash(&user) as u32;

        let groups = t / GROUP_SIZE;
        let offset = seed % groups;

        let mut returner = String::from("BUDDIES");

        for n in 0..GROUP_SIZE {
            returner += &user_list[(offset + n) as usize];
        }

        write_m(sockets.get_mut(&token).unwrap(), returner);
    } else {
        // Send back not found if we don't find the user
        write_m(sockets.get_mut(&token).unwrap(), "404 User Not Found".to_string());
    }

    None
}

pub fn handle_send(
    token: &Token, 
    sockets: &mut SockMap, 
    message: &str, 
    connections: &mut ConnMap, 
    cache: &mut CacheMap
) -> Option<usize> {
    // Pull the sender and receiver out
    let (receiver, orig_message) = message.split_once(";").unwrap();
    let (_, encrypted_message) = orig_message.split_once(";").unwrap();
    let mut message = String::new();

    // Try to find the receiver's struct in connections
    if let Some(user) = connections.get(receiver) {
        // Try to get the stream associated with the user's token
        if let Some(stream) = sockets.get_mut(&user.token) {
            // Send the message
            write_m(stream, "SEND ".to_owned() + orig_message);
            println!("Sent {}", message);
        }

        // Send an ack to the sender as we now take responsibility for delivery
        message = format!("ACK {};{}", receiver, encrypted_message);
        let orig_message = orig_message.to_string();

        // Create a cache or add the message to the receiver's cache in case it is not delivered
        if let Some(v) = cache.get_mut(receiver) {
            v.push(orig_message);
        } else {
            cache.insert(receiver.to_string(), vec![orig_message]);
        }
    } else {
        // If we can't find the receiver, indicate that to the sender
        message = format!("404 {} not found", receiver);
        println!("Sent {}", message);
    }

    // Write the message to the receiver
    write_m(sockets.get_mut(&token).unwrap(), message);
    None
}

pub fn handle_ack(
    message: &str, 
    cache: &mut CacheMap
) -> Option<usize> {
    // Remove the message from the cache one there is a receipt
    let (username, orig_message) = message.split_once(";").unwrap();

    // Get the users cache if it exists (it should always)
    let mut default = Vec::<String>::new();
    let user_cache: &mut Vec<String> = match cache.get_mut(username) {
        Some(v) => v,
        None => &mut default,
    };

    // Try and remove the message by idx from the cache
    if let Some(index) = user_cache.iter().position(|x| *x == orig_message) {
        user_cache.remove(index);
    }

    None
}

pub fn handle_ip_retrieval(
    token: &Token, 
    sockets: &mut SockMap, 
    username: &str, 
    connections: &ConnMap
) -> Option<usize> {
    let message = match connections.get(username) {
        Some(User { token, ip_addr, total_users }) => String::from("IP_RETRIEVAL ") + ip_addr,
        None => String::from("404 not found"),
    };

    write_m(sockets.get_mut(&token).unwrap(), message);
    
    None
}

pub fn handle_error(message: &str) -> Option<usize> {
    println!("received error {}", message);
    None
}

fn write_m(stream: &mut TcpStream, message: String) {
    stream.write_all(message.as_bytes()).unwrap();
    stream.flush().unwrap();
}