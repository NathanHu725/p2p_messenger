use local_ip_address::local_ip;
use std::{io, process, thread};
use threadpool::ThreadPool;
use std::net::{TcpListener, TcpStream};

mod senders;
mod utils;
use handlers::handle_connection;
use senders::{initialize, ip_fetch, send_message};
use utils::{read_file, delete_file};

const PORT: u16 = 8013;

/*
 * Setup a local server and send a "hello" message to the main server
*/

fn setup_server() {
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
            pool.execute(move || {
                handle_connection(&stream);
            });
        }
    });
}

/*
 * The method listens to command line arguments to process user input
*/

fn listen() {
    let commands: &str = "Valid commands: chat [username], clear [username], [message], help, exit";
    println!("Please login by entering the username you would like to use.");
    
    let mut username = String::from("");
    io::stdin().read_line(&mut username).unwrap();
    let username = username.trim();

    // Initialize the connection to the server
    let server: TcpStream = match initialize(&username, &local_ip().unwrap().to_string(), PORT) {
        Some(s) => s,
        None => panic!("Could not connect to server"),
    };

    handle_connection(&server);
    println!("Welcome to Jaelegram\n{}", commands);
    let mut recipient = String::from("");

    loop {
        // Prompt input and process
        let mut answer = String::from("");
        io::stdin().read_line(&mut answer).unwrap();
        let mut answer_tok = answer.split([' ', '\r', '\n']);

        let response = match answer_tok.next().unwrap() {
            "chat" => {
                let user = answer_tok.collect::<Vec<&str>>().join("");
                if user != "" {
                    recipient = String::from(user);
                    read_file(&recipient);
                    Ok(String::from("Entered chat"))
                } else {
                    Err(String::from("Please enter a user"))
                }
            },
            "clear" => {
                let user = answer_tok.collect::<Vec<&str>>().join("");
                if user != "" {
                    recipient = String::from(user);
                    _ = delete_file(&recipient);
                    Ok(String::from("Wiped chat"))
                } else {
                    Err(String::from("Please enter a user"))
                }
            },
            "exit" => {
                process::exit(0);
            },
            "help" => Err(String::from(commands)),
            first_word => {
                if recipient == "" {
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
                    let message = answer_tok.collect::<Vec<&str>>().join(" ");
                    send_message("SEND ".to_owned() + &recipient + ";" + username + ";" + first_word + " " + &message, &server);
                    handle_connection(&server);
                    Ok(String::from("Message Sent"))
                }
            },
        };

        match response {
            Ok(_) => (),
            Err(error) => println!("{}", error),
        }
    }
}


fn main() {
    setup_server();
    listen();
    // read_file("test");


    // let a = [1, 2, 3];

    // let mut doubled = a.iter()
    //                         .map(|&x| x * 2);

    // doubled.next();
    // let rem: Vec<i32> = doubled.collect();

    // assert_eq!(vec![4, 6], rem);
    // println!("{:?}", rem);
}
