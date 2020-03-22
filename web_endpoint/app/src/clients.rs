
use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;


fn main ()  {
    println!("hello world");
    let client = Client::connect("localhost", 27017).expect("Failed to initialize standalone client.");
    let coll = client.db("test").collection("movies");
    let doc = doc!{
        "title": "Jaws",
        "array": [1, 2, 3],
    };

    coll.insert_one(doc.clone(), None).ok().expect("Failed to insert document");
    // Find the document and receive a cursor
    let mut cursor = coll.find(Some(doc.clone()), None)
        .ok().expect("Failed to execute find.");

    let item = cursor.next();

    // cursor.next() returns an Option<Result<Document>>
    match item {
        Some(Ok(doc)) => match doc.get("title") {
            Some(&Bson::String(ref title)) => println!("{}", title),
            _ => panic!("Expected title to be a string!"),
        },
        Some(Err(_)) => panic!("Failed to get next from server!"),
        None => panic!("Server returned no results!"),
    }

}
