#[derive(Debug)]
pub struct Message {
    sender: String,
    receiver: String,
    content: String,
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
        (self.receiver, format!("{}: {}", self.sender, self.content))
    }
}
