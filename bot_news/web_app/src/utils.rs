use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn calculate_hash(input_str: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(input_str.as_bytes());
    hasher.finish()
}
