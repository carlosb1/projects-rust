#[macro_use] extern crate lombok;


#[derive(GetterSetter)]
struct Dog;

fn main() {
    println!("Hello, world!");
    let _dog = Dog{};
}
