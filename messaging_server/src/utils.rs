use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use mio::Token;

/*
 * This struct stores necessary data to identify a user
*/
#[derive(Hash)]
pub struct User {
    pub token: Token,
    pub ip_addr: String,
    pub total_users: u32,
}

/*
 * This has function calculates the hash value of a user
*/

pub fn calculate_hash<User: Hash>(t: &User) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}