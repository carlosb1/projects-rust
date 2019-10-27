#[macro_use] extern crate lombok;


#[derive(GetterSetter)]
struct Dog {
    number: u32,
}

fn main() {
    println!("Hello, world!");
    let _dog = Dog{number: 1};
    _dog.helloworld();
}
