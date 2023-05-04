use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, BufReader, Write};
use chrono::prelude::*;

const MDIR: &str = "./messages/";

#[allow(dead_code)]
pub fn write_message(file_name: String, message: &str) {
    let mut file = match OpenOptions::new()
                        .append(true)
                        .open(file_name.clone()) {
        Ok(file) => file,
        Err(_) => File::create(file_name).unwrap(),
    };
    
    let formatted_t = &Utc::now(). to_rfc2822()[..25];

    // Write the message to the file
    _ = file.write_all((formatted_t.to_owned() + ";" + message + "\n").as_bytes());
}

#[allow(dead_code)]
pub fn read_file(username: &str) {
    println!("Chat with {}", username);
    let file_name: String = MDIR.to_owned() + username + ".txt";
    if let Ok(file) = File::open(file_name) {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                let mut line_tokens = line.split(";");
                println!("{} {} -> {}", line_tokens.next().unwrap(), line_tokens.next().unwrap(), line_tokens.collect::<Vec<_>>().join(";"));
            }
        }
    }
}

#[allow(dead_code)]
pub fn delete_file(username: &str) -> Result<(), std::io::Error> {
    let file_name: String = MDIR.to_owned() + username + ".txt";
    fs::remove_file(file_name)
}