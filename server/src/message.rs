use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub chat_id: i32,
    pub sender_id: i32,
    pub content: String,
}
impl Message {
    pub fn new(chat_id: i32, sender_id: i32, content: String) -> Self {
        Message {
            chat_id,
            sender_id,
            content,
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}
