use local_ip_address::local_ip;
use threadpool::ThreadPool;
use std::{process, thread};
use std::io::{stdin, Read};
use std::net::{TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex, mpsc::{Receiver, channel, TryRecvError}};

mod senders;
mod utils;
use handlers::handle_connection;
use senders::{initialize, ip_fetch, send_message, init_stream};
use utils::{read_file, delete_file};

const PORT: u16 = 8013;
const commands: &str = "Valid commands: chat [username], clear [username], [message], help, exit";

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

    // Keep asking for a new username if ; is used
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

    // Setup listening server once we know who we are
    let server: TcpStream = initialize(&username, &local_ip().unwrap().to_string(), PORT).expect("Could not init connection");

    // Init stdin listener
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
                    "shutdown" => {
                        send_message("SHUTDOWN now".to_owned(), &server);
                        process::exit(0);
                    },
                    "help" => Err(String::from(commands)),
                    _ => {
                        if recipient.lock().unwrap().clone() == "" {
                            Err(String::from("Please enter a conversation first"))
                        } else {
                            // Find the recipient, send the message to the server with them as the target
                            let recip = recipient.lock().unwrap().clone();
                            send_message("SEND ".to_owned() + &recip + ";" + &username + ";" + &answer, &server);
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
                // Handle all connections with the main server in the meantime
                handle_connection(&server, &recipient.lock().unwrap().clone(), &username);
            },
            Err(TryRecvError::Disconnected) => panic!("Channel disconnected"),
        };
    }
}


fn main() {
    let recipient = Arc::new(Mutex::new(String::new()));
    listen(recipient);
}
