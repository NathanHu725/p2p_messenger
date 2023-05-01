use local_ip_address::local_ip;
use threadpool::ThreadPool;
use std::{process, thread};
use std::io::{stdin, Read};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc::{Receiver, channel, TryRecvError}};

mod senders;
mod utils;
use handlers::handle_connection;
use senders::{initialize, ip_fetch, send_message};
use utils::{read_file, delete_file};

const PORT: u16 = 8014;
const commands: &str = "Valid commands: chat [username], clear [username], [message], help, exit";

/*
 * Setup a local server and send a "hello" message to the main server
*/

fn setup_server(recipient: Arc<Mutex<String>>) {
    thread::spawn(move || {
        // Set up TCP listener
        let listener = TcpListener::bind(
            format!("{}:{}", local_ip().unwrap(), PORT)
        ).unwrap();

        // Set up the thread pool
        let num_workers = 4;
        let pool = ThreadPool::new(num_workers);

        // Each message that comes in is passed to the thread pool
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let r_copy = recipient.clone();
            pool.execute(move || {
                handle_connection(&stream, &r_copy.lock().unwrap());
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
 * This method gets the uesrname from stdin
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
 * The method listens to command line arguments to process user input
*/

fn listen(recipient: Arc<Mutex<String>>) {
    println!("Please login by entering the username (no ';') you would like to use:");
    
    // Get the username, check that is doesn't have a ; (our delimiter)
    let mut username = get_username();

    // Initialize the listeners
    let server: TcpStream = initialize(&username, &local_ip().unwrap().to_string(), PORT).expect("Could not init connection");
    println!("{}", commands);
    let stdin = spawn_stdin_channel();

    loop {
        match stdin.try_recv() {
            // If there is stdin input, handle it
            Ok(answer) => {
                let mut answer_tok = answer.split([' ', '\r', '\n']);
                let response = match answer_tok.next().unwrap() {
                    "chat" => {
                        let mut user = answer_tok.collect::<Vec<&str>>().join(" ");
                        user = user.trim().to_string();
                        if user != "" {
                            *recipient.lock().unwrap() = String::from(user);
                            read_file(&recipient.lock().unwrap());
                            Ok(String::from("Entered chat"))
                        } else {
                            Err(String::from("Please enter a user"))
                        }
                    },
                    "clear" => {
                        let user = answer_tok.collect::<Vec<&str>>().join("");
                        if user != "" {
                            _ = delete_file(&user);
                            Ok(String::from("Wiped chat"))
                        } else {
                            Err(String::from("Please enter a user"))
                        }
                    },
                    "exit" => {
                        process::exit(0);
                    },
                    "help" => Err(String::from(commands)),
                    _ => {
                        if recipient.lock().unwrap().clone() == "" {
                            Err(String::from("Please enter a conversation first"))
                        } else {
                            // Get the ip address of the recipient
                            // ip_fetch(&recipient, &server);
                            // let recipient_addr = handle_connection(&server);
    
                            // If found, initialize a connection and send it to the client
                            // if let Some(ip) = recipient_addr {
                            //     let message = answer_tok.collect::<Vec<&str>>().join(" ");
                            //     let message = format!("SEND {};{}", username, message);
    
                            //     match TcpStream::connect(ip) {
                            //         Ok(recip_server) => {
                            //             send_message(message, &recip_server);
                            //             Ok(String::from("Message Sent"))
                            //         },
                            //         Err(_) => Err(String::from("Could not connect to recipient"))
                            //     }
                            // } else {
                            //     Err(String::from("Invalid Recipient"))
                            // }
                            // if let Some(Result(message)) = answer {
                                let recip = recipient.lock().unwrap().clone();
                                send_message("SEND ".to_owned() + &recip + ";" + &username + ";" + &answer, &server);
                                handle_connection(&server, &recip);
                            // }
                            Ok(String::from("Message Sent"))
                        }
                    },
                };
    
                match response {
                    Ok(_) => (),
                    Err(error) => println!("{}", error),
                }
            },
            Err(TryRecvError::Empty) => {
                handle_connection(&server, &recipient.lock().unwrap().clone());
            },
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        };
    }
}


fn main() {
    let recipient = Arc::new(Mutex::new(String::new()));
    setup_server(recipient.clone());
    listen(recipient);
}
