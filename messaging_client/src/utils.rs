use std::fs::{self, File, OpenOptions};
use std::io::{self, prelude::*, BufReader, Write};
use std::time::SystemTime;

const MDIR: &str = "./messages/";

pub fn write_message(file_name: String, message: &str) {
    let mut file = match OpenOptions::new()
                        .append(true)
                        .open(file_name.clone()) {
        Ok(file) => file,
        Err(_) => File::create(file_name).unwrap(),
    };
    
    let curr_t = SystemTime::now();
    let formatted_t = format!("{:?};", curr_t);

    // Write the message to the file
    file.write_all(&[formatted_t.as_bytes(), message.as_bytes()].concat());
}

pub fn read_file(username: &str) {
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