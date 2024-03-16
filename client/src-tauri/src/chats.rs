use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub from_me: bool,
    pub content: String,
    pub status: i32,
}

#[derive(Debug, Deserialize)]
struct ServerMessage {
    chat_id: i32,
    sender_id: i32,
    content: String,
    status: i32,
}

#[derive(Debug)]
pub struct Chats {
    me: i32,
    chats: HashMap<i32, VecDeque<Message>>,
}

impl Chats {
    pub fn new() -> Chats {
        Chats {
            me: i32::default(),
            chats: HashMap::new(),
        }
    }

    pub fn set_id(&mut self, id: i32) {
        self.me = id;
    }

    pub fn add_chat(&mut self, chat_id: i32) {
        if self.chats.contains_key(&chat_id) {
            return;
        }
        self.chats.insert(chat_id, VecDeque::new());
    }

    fn add_message(&mut self, user: i32, content: String, from_me: bool) -> Message {
        let message = Message {
            from_me,
            content,
            status: 1,
        };
        println!("{:?}", self.chats);
        match self.chats.get_mut(&user) {
            Some(chat) => {
                chat.push_back(message.clone());
            }
            None => {
                self.add_chat(user);
            }
        }
        message
    }

    pub fn sent_message(&mut self, user: i32, content: String) -> Message {
        self.add_message(user, content, true)
    }

    pub fn received_message(&mut self, user: i32, content: String) -> Message {
        self.add_message(user, content, false)
    }

    pub fn get_chat(&self, user: i32) -> &VecDeque<Message> {
        println!("{:?}", self.chats.get(&user).unwrap());
        self.chats.get(&user).unwrap()
    }

    pub fn is_me(&self, user: i32) -> bool {
        self.me == user
    }

    pub fn load(&mut self, messages: &str) {
        let messages: Vec<ServerMessage> = serde_json::from_str(messages).unwrap();
        for m in messages.into_iter() {
            self.add_message(m.chat_id, m.content, self.is_me(m.sender_id));
        }
    }
    pub fn my_message_read(&mut self, user: i32) {
        self.set_read(user, true)
    }
    pub fn other_message_read(&mut self, user: i32) {
        self.set_read(user, false)
    }
    fn set_read(&mut self, user: i32, from_me: bool) {
        for m in self.chats.get_mut(&user).unwrap().iter_mut().rev() {
            if m.from_me != from_me {
                continue;
            }
            if m.status == 2 {
                break;
            }
            m.status = 2;
        }
    }
}
