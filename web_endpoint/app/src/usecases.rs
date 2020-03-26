use crate::db::{UsersRepository, ChannelsRepository};
use crate::entities::{User, Channel};
use std::sync::Arc;

pub struct  GetChannel {
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

pub struct  GetUser {
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

struct  NewChannel {
    db: Box<ChannelsRepository>
}

impl NewChannel{
    pub fn new(db: Box<ChannelsRepository>)->NewChannel {
        NewChannel{db: db}
    }
    pub fn run(self, channel: Channel) {
        //unwrap shared reference
        self.db.create(channel)    
    }
}

struct  NewUser {
    db: Box<UsersRepository>
}

impl NewUser{
    pub fn new(db: Box<UsersRepository>)->NewUser {
        NewUser{db: db}
    }
    pub fn run(self, user: User) {
        //unwrap shared reference
        self.db.create(user) 
    }
}



