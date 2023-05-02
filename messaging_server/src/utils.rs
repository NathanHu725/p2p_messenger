use std::hash::{Hash, Hasher};
use mio::Token;

/*
 * This struct stores necessary data to identify a user
*/
#[derive(Hash)]
struct User {
    token: Token,
    ip_addr: String,
    total_users: u32,
}

struct 

/*
 * This has function calculates the hash value of a user
*/

fn calculate_hash<T: Hash>(t: &()) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}