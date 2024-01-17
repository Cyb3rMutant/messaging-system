use std::collections::{HashMap, VecDeque};

use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub from: String,
    pub content: String,
}

#[derive(Debug)]
pub struct Chats {
    me: String,
    chats: HashMap<String, VecDeque<Message>>,
}

impl Chats {
    pub fn new() -> Chats {
        Chats {
            me: String::new(),
            chats: HashMap::new(),
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.me = name;
    }

    pub fn add_chat(&mut self, user: String) {
        if self.chats.contains_key(&user) {
            return;
        }
        self.chats.insert(user, VecDeque::new());
    }

    fn add_message(&mut self, user: &str, content: String, from_me: bool) -> Message {
        let message = Message {
            from: if from_me {
                self.me.clone()
            } else {
                user.to_owned()
            },
            content,
        };
        self.chats.get_mut(user).unwrap().push_back(message.clone());
        message
    }

    pub fn sent_message(&mut self, user: &str, content: String) -> Message {
        self.add_message(user, content, true)
    }

    pub fn received_message(&mut self, user: &str, content: String) -> Message {
        self.add_message(user, content, false)
    }

    pub fn get_chat(&self, user: &str) -> &VecDeque<Message> {
        self.chats.get(user).unwrap()
    }

    pub fn is_me(&self, user: &str) -> bool {
        self.me == user
    }
}
