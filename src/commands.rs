use crate::{client::Client, message::Message};

#[derive(Debug)]
pub enum Command {
    Add { client: Client },
    Send { message: Message },
    Remove { name: String },
}
