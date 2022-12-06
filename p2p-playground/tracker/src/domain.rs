use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message class for messages. It is serialize in a json message.
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Message {
    pub operation: String,
    pub topic: String,
    pub info: HashMap<String, String>,
    pub mesg: String,
}

impl Message {
    pub fn new(operation: String) -> Message {
        Message {
            operation,
            ..Default::default()
        }
    }

    pub fn new_user(user: String, address_source: String) -> Message {
        let mut info: HashMap<String, String> = HashMap::new();
        info.insert(user, address_source);
        Message {
            operation: "ack".to_string(),
            ..Default::default()
        }
    }
}

pub struct User {
    pub username: String,
    pub source_address: String,
}

impl User {
    pub fn new(username: String, source_address: String) -> Self {
        User {
            username,
            source_address,
        }
    }
}

pub fn add_new_user(user_repo: &mut HashMap<String, User>, user: User) {
    (*user_repo).insert(user.username.clone(), user);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[rstest]
    fn should_add_a_new_user() {
        let mut user_repo = HashMap::new();
        let new_user = User::new("user1".to_string(), "192.168.1.1".to_string());
        add_new_user(&mut user_repo, new_user);
        assert_eq!(user_repo.len(), 1);
    }
}
