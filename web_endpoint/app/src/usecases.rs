use crate::db::{UsersRepository, ChannelsRepository};
use crate::entities::{User, Channel};

pub struct GetChannel {
    db: ChannelsRepository
}

impl GetChannel{
    pub fn new(db: ChannelsRepository)->GetChannel {
        GetChannel{db: db}
    }
    pub fn run(self, idchannel: String) -> Option<Channel> {
        //unwrap shared reference
        self.db.get(idchannel)    
    }
}

pub struct GetUser {
    db: Box<UsersRepository>
}

impl GetUser{
    pub fn new(db: Box<UsersRepository>)->GetUser {
        GetUser{db: db}
    }
    pub fn run(self, idname: String) -> Option<User> {
        //unwrap shared reference
        self.db.get(idname)    
    }
}

pub struct  NewChannel {
    db: Box<ChannelsRepository>
}

impl NewChannel{
    pub fn new(db: Box<ChannelsRepository>)->NewChannel {
        NewChannel{db: db}
    }
    pub fn run(self, channel: Channel) {
        //unwrap shared reference
        self.db.create(channel); 
    }
}

pub struct  NewUser {
    db: Box<UsersRepository>
}

impl NewUser{
    pub fn new(db: Box<UsersRepository>)->NewUser {
        NewUser{db: db}
    }
    pub fn run(self, user: User) {
        //unwrap shared reference
        self.db.create(user); 
    }
}

pub struct UpdateUser {
    db: Box<UsersRepository>
}

impl UpdateUser {
    pub fn new (db: Box<UsersRepository>) -> UpdateUser  {
            UpdateUser{db: db}
        }
    pub fn run (self, user: User) {
        self.db.put(user); 
    }
}


pub struct UpdateChannel {
    db: Box<ChannelsRepository>
}

impl UpdateChannel {
    pub fn new (db: Box<ChannelsRepository>) -> UpdateChannel  {
            UpdateChannel{db: db}
        }
    pub fn run (self, channel: Channel) {
        self.db.put(channel); 
    }
}

