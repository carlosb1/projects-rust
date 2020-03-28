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
    db: UsersRepository
}

impl GetUser{
    pub fn new(db: UsersRepository)->GetUser {
        GetUser{db: db}
    }
    pub fn run(self, idname: String) -> Option<User> {
        //unwrap shared reference
        self.db.get(idname)    
    }
}

pub struct  NewChannel {
    db: ChannelsRepository
}

impl NewChannel{
    pub fn new(db: ChannelsRepository)->NewChannel {
        NewChannel{db: db}
    }
    pub fn run(self, channel: Channel) {
        //unwrap shared reference
        self.db.create(channel); 
    }
}

pub struct  NewUser {
    db: UsersRepository
}

impl NewUser{
    pub fn new(db: UsersRepository)->NewUser {
        NewUser{db: db}
    }
    pub fn run(self, user: User) {
        //unwrap shared reference
        self.db.create(user); 
    }
}

pub struct UpdateUser {
    db: UsersRepository
}

impl UpdateUser {
    pub fn new (db: UsersRepository) -> UpdateUser  {
            UpdateUser{db: db}
        }
    pub fn run (self, user: User) {
        self.db.put(user); 
    }
}


pub struct UpdateChannel {
    db: ChannelsRepository
}

impl UpdateChannel {
    pub fn new (db: ChannelsRepository) -> UpdateChannel  {
            UpdateChannel{db: db}
        }
    pub fn run (self, channel: Channel) {
        self.db.put(channel); 
    }
}

