use std::collections::HashMap;


struct Chat<'a> {
    channels: HashMap<&'a str, Vec<&'a str>>,
}


impl<'a> Chat<'a> {
    fn new() -> Chat<'a> {
        Chat{channels: HashMap::new()} 
    }

    fn join(&mut self, name_channel: &'a str, username: &'a str){
        self.channels.entry(name_channel).or_insert_with(Vec::new).push(username);
    }
    fn remove(&mut self, name_channel: &'a str, username: &'a str) {
        let mut elems = self.channels.get_mut(name_channel).unwrap();
        elems.iter().position(|&x| x == username).map(|e| elems.remove(e));
    }
}



pub fn main () -> Result<(), Box<std::error::Error>> {
    let mut  chat = Chat::new(); 
    chat.join("tech", "juan");
    chat.join("tech", "jose");
    chat.remove("tech","juan");
   
    /* */
    for (k, v) in chat.channels {
        println!("{}", k);
        for ve in v {
            println!("vec={}", ve);
        }
    }

    Ok(())
}
