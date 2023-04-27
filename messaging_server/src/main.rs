use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use local_ip_address::local_ip;
// use mio::net::TcpStream;
use mio::{Events, Poll, Token, Interest};
use std::net::{TcpListener, TcpStream};
use std::{time, thread};
use threadpool::ThreadPool;

use handlers::{handle_connection, CacheMap, ConnMap};

const PORT: u16 = 8013;

fn setup_server(conn: ConnMap, 
                cache: CacheMap) {
    // Create poll and appropriate objects
    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    // thread::spawn(move || {
    let listener = TcpListener::bind(
        format!("{:?}:{}", local_ip().unwrap(), PORT)
    ).unwrap();
    listener.set_nonblocking(true).expect("Cannot set nonblocking");

    // Set up the thread pool
    let num_workers = 4;
    let pool = ThreadPool::new(num_workers);

    // Each message that comes in is passed to the thread pool
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let temp_conn = conn.clone();
                let temp_cache = cache.clone();
                // pool.execute(move || {
                //     handle_connection("", s, temp_conn, temp_cache);
                // });
                poll.register().register(&smut , Token(0), Interest READABLE | Interest::WRITEABLE);
                println!("Found connection");
                // 
            },
            Err(_) =>  {
                poll.poll(&mut events, None).unwrap();
                for event in &events {
                    match event.token() {
                        Token(0) => {
                            println!("Found token");
                        },
                        _ => {
                            println!("Found something else");
                        }
                    }
                }
            },
        }
    }
    // });

    // let conn = conn.clone();
    // let cache = cache.clone();

    // loop {
    //     let poll_bind = match poll.lock() {
    //         Ok(v) => v,
    //         Err(_) => panic!("No Guard"),
    //     };
    //     let b = match &*poll_bind {
    //         Ok(v) => v,
    //         Err(_) => panic!("No Guard"),
    //     };
    //     // Set up the thread pool
    //     // let num_workers = 4;
    //     // let pool = ThreadPool::new(num_workers);

    //     // for (username, stream) in conn.lock().unwrap().iter() {
    //     //     let stream = stream.try_clone().expect("failed");
    //     //     let mut buf = [0; 10];
    //     //     if let Ok(something) = stream.peek(&mut buf) {
    //     //         let username = username.clone();
    //     //         println!("Found 1");
    //     //         let temp_conn = conn.clone();
    //     //         let temp_cache = cache.clone();
    //     //         pool.execute(move || {
    //     //             handle_connection(&username, stream, temp_conn, temp_cache);
    //     //         });
    //     //     }
    //     // };
    // }
}


fn main() {
    let active_connections = Arc::new(Mutex::new(HashMap::new()));
    let cached_messages = Arc::new(Mutex::new(HashMap::new()));
    setup_server(active_connections.clone(), cached_messages.clone());

    println!("Hello, world!");
}