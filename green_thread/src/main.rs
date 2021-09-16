#![feature(llvm_asm)]

const SSIZE: isize = 48;

#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64,
}

fn main() {
    println!("Hello, world!");
}
