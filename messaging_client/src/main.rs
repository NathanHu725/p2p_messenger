use local_ip_address::local_ip;
use std::collections::{HashMap, VecDeque};
use std::io::stdin;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{
    mpsc::{channel, Receiver, TryRecvError},
    Arc, Mutex,
};
use std::{process, thread};
use threadpool::ThreadPool;

use lib::network_messaging::handlers::{handle_ack, handle_connection, handle_ip_retrieval, handle_main_server_connection, MDIR};
use lib::network_messaging::senders::{
    init_stream, initialize, ip_fetch, send_backups, send_message,
};
use lib::network_messaging::utils::{delete_file, read_file, write_message, Job, RING_SIZE, NUM_FINGERS, MAX_GROUP_SIZE};

const PORT: u16 = 8013;
const COMMANDS: &str = "Valid commands: chat [username], clear [username], [message], help, exit";

/*
 * Setup a local server and send a "hello" message to the main server
*/

fn setup_server(recipient: Arc<Mutex<String>>, username: String, event_queue: Arc<Mutex<VecDeque<String>>>, tx: std::sync::mpsc::Sender<Job>) {
    thread::spawn(move || {
        // Set up TCP listener
        let listener = TcpListener::bind(format!("{}:{}", local_ip().unwrap(), PORT)).unwrap();

        // Set up the cache, only to be accessed by the server thread
        let cache = Arc::new(Mutex::new(HashMap::new()));

        // The group index
        let mut group_index = 0;

        // Set up the finger list
        let mut fingers: Arc<Mutex<[(String, String); NUM_FINGERS]>> = Arc::new(Mutex::new([(, "".to_string()); NUM_FINGERS]));

        // Set up the vector that stores all group members
        let mut group_members = Arc::new(Mutex::new(Vec::with_capacity(MAX_GROUP_SIZE)));

        // Local list of addresses we have cached
        let mut cached_addresses = Arc::new(Mutex::new(VecDeque::new()));

        // Local storage of user structs
        let mut storage = Arc::new(Mutex::new(HashMap::new()));

        // Set up the thread pool
        let num_workers = 8;
        let pool = ThreadPool::new(num_workers);

        // Each message that comes in is passed to the thread pool
        for stream in listener.incoming() {
            let stream = stream.unwrap();

            // Read the message into a buffer
            let mut buffer = [0; 2048];

            if let Ok(i) = stream.read(&mut buffer) {
                let as_string = std::str::from_utf8(&buffer[..i]).unwrap();
        
                // Handle based on the status code
                if let Some((code, message)) = as_string.split_once(" ") {
                    let response: HandlerResult = match code {
                        "IP_FETCH" => {
                            let addr_copy = cached_addresses.clone();
                            let stor_copy = storage.clone();
                            let f_copy = fingers.clone();


                        },
                        "IP_RETRIEVAL" => {

                        },
                        "INIT" => {
                            handle_init(message, cache, &group_index),
                        },
                        "SEND" => {
                            // Check if the message is for us or not, if not propagate to cache
                            // This can accept ACKS as well
                            let mut cache_copy = cache.clone();
                            handle_send(message, recip, user),
                        },
                        "CACHE" => {
                            // Cache this message, do not propagate
                        },
                        "UPDATE_FINGERS" => {
                            // Update the finger list
                        },
                        "NEW_FINGER" => {
                            let group_copy = group_members.clone();
                        },
                        "UPDATE_GROUP" => {

                        },
                        "404" => handle_not_found(message),
                        _ => handle_error(message),
                    };
                };
            }
        }
    });
}

/*
 * This method gets the username from stdin
*/

fn get_username() -> String {
    let mut username = String::from("");
    stdin().read_line(&mut username).unwrap();
    while username.contains(";") {
        println!("No ';' characters allowed");
        stdin().read_line(&mut username).unwrap();
        username.clear();
    }
    username.trim().to_string()
}

/*
 * The method listens to command line arguments to process user input and pass
 * it through appropriate channels
*/

fn main() {
    println!("Please login by entering the username (no ';') you would like to use:");

    // Get the username, check that is doesn't have a ; (our delimiter)
    let username = get_username();

    // Setup listening server once we know who we are
    let mut server: TcpStream = initialize(&username, &local_ip().unwrap().to_string(), PORT)
        .expect("Couldn't connect to the gateway server");
    
    // Setup shared server vars
    let recipient = Arc::new(Mutex::new(String::new()));
    let event_queue = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = channel::<Job>();

    setup_server(recipient.clone(), username.clone(), event_queue.clone(), tx);

    // Init stdin listener
    println!("{}", COMMANDS);

    loop {
        let mut buffer = String::new();
        let input = stdin().read_line(&mut buffer).unwrap();
        let mut answer_tok = input.split([' ', '\r', '\n']);
        let response = match answer_tok.next().unwrap() {
            "chat" => {
                // Switch the chat to the input user
                let mut user = answer_tok.collect::<Vec<&str>>().join(" ");
                user = user.trim().to_string();

                if user != "" {
                    // Print the record of the chat with that user
                    read_file(&user);
                    *recipient.lock().unwrap() = user;
                    Ok(String::from("Entered chat"))
                } else {
                    // Prompt for a user if not entered
                    Err(String::from("Please enter a user"))
                }
            }
            "clear" => {
                // Find the user based on input
                let user = answer_tok.collect::<Vec<&str>>().join("");
                if user != "" {
                    // Delete file with the record
                    _ = delete_file(&user);
                    Ok(String::from("Wiped chat"))
                } else {
                    // Prompt for a user if not entered
                    Err(String::from("Please enter a user"))
                }
            }
            "exit" => {
                // Graceful exit
                process::exit(0);
            }
            "help" => Err(String::from(COMMANDS)),
            _ => {
                let recip_copy = recipient.lock().unwrap().clone();
                // All other strings are interpreted as messages meant to be sent
                if recip_copy == "" {
                    // If not in a convo, require that first
                    Err(String::from("Please enter a conversation first"))
                } else {
                    // Treat the send input as requried by the method
                    rx.send(Job::new(recip_copy.clone(), username.clone(), input.trim().to_string()));
                }
            }
        };

        match response {
            Ok(_) => (),
            Err(error) => println!("{}", error),
        }
    }
}