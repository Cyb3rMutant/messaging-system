use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub sender: String,
    pub receiver: String,
    pub content: String,
}
impl Message {
    pub fn new(sender: String, receiver: String, content: String) -> Self {
        Message {
            sender,
            receiver,
            content,
        }
    }
    pub fn parse(self) -> (String, String) {
        (
            self.receiver,
            format!("MSG;{};{}", self.sender, self.content),
        )
    }

    pub fn get_users(&self) -> (&str, &str) {
        (&self.sender, &self.receiver)
    }

    pub fn get_content(&self) -> &str {
        &self.content
    }
}
