
#[derive(Clone)]    
pub struct User  {
    pub idname: String,
    pub idaddress: String,
}
#[derive(Clone)]    
pub struct Channel {
    pub name: String,
    pub users: Vec<String>,
}
