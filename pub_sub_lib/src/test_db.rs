use rocksdb::{DB, Options};
use serde_json::Result;
use std::collections::HashMap;

pub struct DBRepository {
    filepath: String
}

impl DBRepository {
    fn new(filepath: String) -> DBRepository {
        DBRepository{filepath: filepath}
    }

    fn save(self, key: String,  info: HashMap<String, String>){
        let parsed_info = serde_json::to_string(&info).unwrap();
        let db = DB::open_default(self.filepath).unwrap();
        db.put(key, parsed_info).unwrap(); 
    }

    fn get(self, key: String) -> Option<HashMap<String, String>> {
        let db = DB::open_default(self.filepath).unwrap();
        let ret =  match db.get(key.clone()) {
            Ok(Some(value)) =>  {
                let tmp_val = String::from_utf8(value).unwrap();
                let str_result = tmp_val.as_str();
                Some(serde_json::from_str(str_result).unwrap())
                },
            Ok(None) =>  None,
            Err(e) =>{ println!("operational problem encountered: {}", e); None},
        };
        db.delete(key); 
        ret
    }
}

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
