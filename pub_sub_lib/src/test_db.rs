use rocksdb::{DB, Options};
use serde_json::Result;

pub struct DBRepository;

/*
impl DBRepository {

}
*/

fn main () {
    let mut info: Vec<String> = Vec::new();
    info.push("my".to_string());
    info.push("world".to_string());
    let vec_str = serde_json::to_string(&info).unwrap();
    println!("Hello test {}?", vec_str);
    let path = "rocksdbstorage";
    {
        let db = DB::open_default(path).unwrap();
        db.put(b"my key", vec_str).unwrap();
    
        match db.get(b"my key") {
            Ok(Some(value)) =>  println!("retrieved value {}", String::from_utf8(value).unwrap()),
            Ok(None) => println!("value not found"),
            Err(e) => println!("operational problem encountered: {}", e),
        }
        db.delete(b"my key");
    }
    //let _ = DB::destroy(&Options::default(), path);
}
