use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub from_me: bool,
    pub content: String,
}

// impl<'de> Deserialize<'de> for Message {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let value: Value = Deserialize::deserialize(deserializer)?;
//
//         // Ensure the input is a JSON object
//         let obj = match value {
//             Value::Object(obj) => obj,
//             _ => return Err(serde::de::Error::custom("Expected a JSON object")),
//         };
//
//         // Extract values from the JSON object
//         let sender = obj.get("sender").and_then(|v| v.as_str());
//         let content = obj.get("content").and_then(|v| v.as_str());
//
//         // Check if required fields are present
//         let sender = sender.ok_or_else(|| serde::de::Error::missing_field("sender"))?;
//         let content = content.ok_or_else(|| serde::de::Error::missing_field("content"))?;
//
//         // Initialize the Message struct
//         Ok(Message {
//             from_me: sender.parse().unwrap(),
//             content: content.to_string(),
//         })
//     }
// }
#[derive(Debug, Deserialize)]
struct ServerMessage {
    chat_id: i32,
    sender_id: i32,
    content: String,
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
        let message = Message { from_me, content };
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
}
