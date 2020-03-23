/*
struct  {
    db: Box<dyn DBAdapter>
}

impl GetPostsCase{
    pub fn new(db: Box<dyn DBAdapter>)->GetPostsCase {
        GetPostsCase{db: db}
    }
    pub fn run(&self) -> Vec<Post> {
        //unwrap shared reference
        let result = self.db.read();
        result   
    }
}
*/
