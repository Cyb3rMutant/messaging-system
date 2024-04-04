use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub message_id: i32,
    pub chat_id: i32,
    pub sender_id: i32,
    pub content: String,
    pub status: i32,
}
impl Message {
    pub fn new(chat_id: i32, sender_id: i32, content: String, status: i32) -> Self {
        Message {
            message_id: 0,
            chat_id,
            sender_id,
            content,
            status,
        }
    }
    pub fn update(
        message_id: i32,
        chat_id: i32,
        sender_id: i32,
        content: String,
        status: i32,
    ) -> Self {
        Message {
            message_id,
            chat_id,
            sender_id,
            content,
            status,
        }
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}
