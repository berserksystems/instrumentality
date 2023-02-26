use std::fmt::Write;

use sha2::{Digest, Sha256};

pub fn new_key() -> (String, String) {
    new_rand_string(32)
}

pub fn new_invite_code() -> (String, String) {
    new_rand_string(64)
}

pub fn hash_string(string: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(string);
    let hashed = hasher.finalize();
    let hashed_hex_string = bytes_to_hex_string(&hashed);
    hashed_hex_string
}

fn new_rand_string(length: usize) -> (String, String) {
    let mut bytes = vec![0; length];
    getrandom::getrandom(&mut bytes).unwrap();

    let hex_string = bytes_to_hex_string(&bytes);

    let hashed_hex_string = hash_string(&hex_string);
    (hex_string, hashed_hex_string)
}

fn bytes_to_hex_string(bytes: &[u8]) -> String {
    let mut code = String::new();
    for b in bytes {
        write!(&mut code, "{b:0>2X}").unwrap();
    }
    code
}
