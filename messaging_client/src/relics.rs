/*
 * THIS IS ALL DEPRECATED CODE KEPT AROUND FOR REFERENCE
*/

/*
 * This method takes an input that is supposed to be sent and handles it appropriately
*/

// fn send_input(
//     recip: &str,
//     server: &mut TcpStream,
//     username: &str,
//     input: &str,
// ) -> Result<String, String> {
//     // Ask for the ip_address of the recipient
//     ip_fetch(recip, server);
//     _ = server.set_nonblocking(false);

//     // Search for the user, send directly if they are online, otherwise to their cache
//     if let Some(ip_addr) = handle_ip_retrieval(server) {
//             // If the user exists, try to send the message directly to them
//             if let Ok(mut stream) = init_stream(&ip_addr) {
//                 // If we can connect to the user, send the message directly to them
//                 send_message(&["SEND ".as_bytes(), username.as_bytes(), ";".as_bytes(), input.as_bytes()].concat(), &stream);
//                 handle_ack(&mut stream, recip);
//                 _ = stream.shutdown(Shutdown::Both);
//             } else {
//                 // Otherwise, send the message to the server to be cached
//                 match send_backups(recip, username, input, server) {
//                     Some(_) => write_message(MDIR.to_owned() + recip + ".txt", &("You;".to_owned() + input)),
//                     None => println!("Message not sent"),
//                 };
//             }
//     } else {
//         // User was not found
//         println!("{} not found", recip);
//     }

//     _ = server.set_nonblocking(true);
//     Ok(String::from("Message Sent"))
// }

/*
 * This method sets up the thread that listens to the input stream
*/

// fn spawn_stdin_channel() -> Receiver<String> {
//     let (tx, rx) = channel::<String>();
//     thread::spawn(move || loop {
//         let mut buffer = String::new();
//         stdin().read_line(&mut buffer).unwrap();
//         tx.send(buffer).unwrap();
//     });
//     rx
// }