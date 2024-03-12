use sha2::{Sha256, Digest};

pub fn hash_from_string(input: String) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}

pub fn hash_from_u8(input: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}
