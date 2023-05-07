use local_ip_address::local_ip;
use std::io::stdin;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{
    mpsc::{channel, Receiver, TryRecvError},
    Arc, Mutex,
};
use std::{process, thread};
use threadpool::ThreadPool;

use lib::network_messaging::handlers::handle_connection;
use lib::network_messaging::senders::{
    init_stream, initialize, ip_fetch, send_backups, send_message,
};
use lib::network_messaging::utils::{delete_file, read_file};

const PORT: u16 = 8013;
const COMMANDS: &str = "Valid commands: chat [username], clear [username], [message], help, exit";

/*
 * Setup a local server and send a "hello" message to the main server
*/

fn setup_server(recipient: Arc<Mutex<String>>, username: String) {
    thread::spawn(move || {
        // Set up TCP listener
        let listener = TcpListener::bind(format!("{}:{}", local_ip().unwrap(), PORT)).unwrap();

        // Set up the thread pool
        let num_workers = 4;
        let pool = ThreadPool::new(num_workers);

        // Each message that comes in is passed to the thread pool
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let r_copy = recipient.clone();
            let user = username.clone();
            pool.execute(move || {
                handle_connection(&stream, &r_copy.lock().unwrap(), &user);
            });
        }
    });
}

/*
 * This method sets up the thread that listens to the input stream
*/

fn spawn_stdin_channel() -> Receiver<String> {
    let (tx, rx) = channel::<String>();
    thread::spawn(move || loop {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        tx.send(buffer).unwrap();
    });
    rx
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
 * This method takes an input that is supposed to be sent and handles it appropriately
*/

fn send_input(recip: &str, server: &mut TcpStream, username: &str, input: &str) -> Result<String, String> {
    // Ask for the ip_address of the recipient
    ip_fetch(recip, server);
    _ = server.set_nonblocking(false);

    // Search for the user, send directly if they are online, otherwise to their cache
    if let Some(addr_result) = handle_connection(server, recip, username) {
        if let Ok(ip_addr) = addr_result {
            // If the user exists, try to send the message directly to them
            if let Ok(mut stream) = init_stream(ip_addr) {
                // If we can connect to the user, send the message directly to them
                send_message("SEND ".to_owned() + username 
                                + ";" + input, stream);
                handle_connection(stream, recip, username);
                _ = stream.shutdown(Shutdown::Both);
            } else {
                // Otherwise, send the message to the server to be cached
                match send_backups(recip, username, input, server) {
                    Some(_) => write_message(recip, &("You;".to_owned() + input));,
                    None => println!("Message not sent"),
                };
            }
        } else {
            // User was not found
            println!("{} not found", recip);
        }
    }
    _ = server.set_nonblocking(true);
    Ok(String::from("Message Sent"))
}

/*
 * The method listens to command line arguments to process user input and pass
 * it through appropriate channels
*/

fn listen(recipient: Arc<Mutex<String>>) {
    println!("Please login by entering the username (no ';') you would like to use:");

    // Get the username, check that is doesn't have a ; (our delimiter)
    let username = get_username();

    // Setup listening server once we know who we are
    let server: TcpStream = initialize(&username, &local_ip().unwrap().to_string(), PORT)
        .expect("Could not init connection");
    setup_server(recipient.clone(), username.clone());

    // Init stdin listener
    println!("{}", COMMANDS);
    let stdin = spawn_stdin_channel();

    loop {
        match stdin.try_recv() {
            // If there is stdin input, handle it according to the command
            Ok(input) => {
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
                    "shutdown" => {
                        // This allows remote shutdown of the server - added for ease of use
                        send_message("SHUTDOWN now".to_owned(), &server);
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
                            send_input(&recip_copy, &server, &username, &input)
                        }
                    }
                };

                match response {
                    Ok(_) => (),
                    Err(error) => println!("{}", error),
                }
            }
            Err(TryRecvError::Empty) => {
                // Handle all connections with the main server in the meantime
                handle_connection(&server, &recipient.lock().unwrap().clone(), &username);
            }
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        };
    }
}

fn main() {
    let recipient = Arc::new(Mutex::new(String::new()));
    listen(recipient);
}
