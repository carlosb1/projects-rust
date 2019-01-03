#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: i32,
    y: i32,
}

fn main () {
     let point = Point { x: 1, y: 2};
     let serialized = serde_json::to_string(&point).unwrap();

     println!("serialized = {}", serialized);

     let bytes = serialized.as_bytes(); 
     let extracted_serialized = String::from_utf8_lossy(bytes);
     let deserialized: Point = serde_json::from_str(&extracted_serialized).unwrap();
    println!("deserialized = {:?}", deserialized);
}

