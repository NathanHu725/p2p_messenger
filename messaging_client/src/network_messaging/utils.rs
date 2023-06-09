use chrono::prelude::*;
use std::fs::{self, File, OpenOptions};
use std::io::{prelude::*, BufReader, Write};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const MDIR: &str = "./messages/";
pub const RING_SIZE: u32 = 4096;
pub const NUM_FINGERS: u32 = 12;
pub const MAX_GROUP_SIZE: u32 = 20;

/*
 * This struct stores necessary data to identify a user
*/
#[derive(Hash)]
pub struct User {
    pub ip_addr: String,
    pub total_users: u32,
}

/*
 * This struct stores necessary data about an event job
*/
pub struct Job {
    pub prefix: String,
    pub message: String,
    pub receipient: String,
}

/*
 * This has function calculates the hash value of a user
*/

pub fn calculate_hash<User: Hash>(t: &User) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

/*
 * Write a message to a file, creates a new file if one doesn't exist
*/

#[allow(dead_code)]
pub fn write_message(file_name: String, message: &str) {
    let mut file = match OpenOptions::new().append(true).open(file_name.clone()) {
        Ok(file) => file,
        Err(_) => File::create(file_name).unwrap(),
    };

    let formatted_t = &Utc::now().to_rfc2822()[..25];

    // Write the message to the file
    _ = file.write_all((formatted_t.to_owned() + ";" + message + "\n").as_bytes());
}

/*
 * Reads from a file if it exists - formats appropriately
*/

#[allow(dead_code)]
pub fn read_file(username: &str) {
    println!("Chat with {}", username);
    let file_name: String = MDIR.to_owned() + username + ".txt";
    if let Ok(file) = File::open(file_name) {
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                let mut line_tokens = line.split(";");
                println!(
                    "{} {} -> {}",
                    line_tokens.next().unwrap(),
                    line_tokens.next().unwrap(),
                    line_tokens.collect::<Vec<_>>().join(";")
                );
            }
        }
    }
}

/*
 * Deletes a file if it exists
*/

#[allow(dead_code)]
pub fn delete_file(username: &str) -> Result<(), std::io::Error> {
    let file_name: String = MDIR.to_owned() + username + ".txt";
    fs::remove_file(file_name)
}
