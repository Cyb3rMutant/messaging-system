use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub from: String,
    pub content: String,
}

impl<'de> Deserialize<'de> for Message {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: Value = Deserialize::deserialize(deserializer)?;

        // Ensure the input is a JSON object
        let obj = match value {
            Value::Object(obj) => obj,
            _ => return Err(serde::de::Error::custom("Expected a JSON object")),
        };

        // Extract values from the JSON object
        let sender = obj.get("sender").and_then(|v| v.as_str());
        let content = obj.get("content").and_then(|v| v.as_str());

        // Check if required fields are present
        let sender = sender.ok_or_else(|| serde::de::Error::missing_field("sender"))?;
        let content = content.ok_or_else(|| serde::de::Error::missing_field("content"))?;

        // Initialize the Message struct
        Ok(Message {
            from: sender.to_string(),
            content: content.to_string(),
        })
    }
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
        println!("{:?}", self.chats.get(user).unwrap());
        self.chats.get(user).unwrap()
    }

    pub fn is_me(&self, user: &str) -> bool {
        self.me == user
    }

    // pub fn load(&self, messages: &str) {
    //     let messages: Vec<Message> = serde_json::from_str(messages).unwrap();
    //     for m in messages.into_iter() {
    //         if self.is_me(m.from) {
    //
    //         }
    //     }
    // }
}
