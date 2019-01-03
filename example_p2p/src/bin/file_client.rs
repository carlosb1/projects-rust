extern crate crypto; 

use self::crypto::digest::Digest;
use self::crypto::sha1::Sha1;

use std::io;
use std::io::prelude::*;
use std::fs::File;


fn main () -> io::Result<()> {
    // read file buffer
    let filename = String::from("/home/carlosb/Desktop/paper-ton.pdf");
    let mut fil = File::open(filename)?;
    let mut buffer = Vec::new();
    fil.read_to_end(&mut buffer)?;

    // calculate sha1
    let mut hasher = Sha1::new();
    hasher.input_str("hello world");
    let hex = hasher.result_str(); 
    println!("my hash = {}", hex);
    hasher.reset();
    hasher.input(&mut buffer);
    let hex = hasher.result_str();
    println!("my hash2 = {}", hex);

    Ok(())
}
