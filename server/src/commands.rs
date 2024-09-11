use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    sync::oneshot,
};

use crate::message::Message;

pub enum GetTypes {
    ALL,
    PENDING,
    FRIENDS,
    BLOCKED,
}

pub enum Command<T: AsyncWriteExt> {
    Add {
        name_pass: String,
        writer: WriteHalf<T>,
        sender: oneshot::Sender<Result<i32, WriteHalf<T>>>,
    },
    Register {
        name_pass: String,
        sender: oneshot::Sender<bool>,
    },
    Connect {
        id: i32,
        other: i32,
    },
    Block {
        id: i32,
        other: i32,
    },
    Unblock {
        id: i32,
        other: i32,
    },
    Send {
        message: Message,
    },
    Remove {
        id: i32,
    },
    GET {
        t: GetTypes,
        id: i32,
    },
    Status {
        chat_id: i32,
        id: i32,
    },
    Delete {
        chat_id: i32,
        id: i32,
        message_id: i32,
    },
    Update {
        message: Message,
    },

    Testing_Clear,
}

impl<T: AsyncWriteExt> Command<T> {
    pub fn send(content: &str, id: i32) -> Result<Command<T>, ()> {
        match content.split_once(';') {
            Some((chat_id, message)) => Ok(Command::Send {
                message: Message::new(chat_id.parse().unwrap(), id, message.to_owned(), 1),
            }),
            _ => Err(()),
        }
    }
    pub fn status(content: &str, id: i32) -> Result<Command<T>, ()> {
        Ok(Command::Status {
            chat_id: content.parse().unwrap(),
            id,
        })
    }

    pub fn delete(content: &str, id: i32) -> Result<Command<T>, ()> {
        let (chat_id, message_id) = content.split_once(';').unwrap();
        println!("{chat_id:?} {message_id:?}");
        Ok(Command::Delete {
            chat_id: chat_id.parse().unwrap(),
            id,
            message_id: message_id.parse().unwrap(),
        })
    }
    pub fn update(content: &str, id: i32) -> Result<Command<T>, ()> {
        let args: Vec<&str> = content.split(';').collect();
        let chat_id = args.get(0).unwrap().parse().unwrap();
        let message_id = args.get(1).unwrap().parse().unwrap();
        let content = args.get(2).unwrap();
        Ok(Command::Update {
            message: Message::update(message_id, chat_id, id, content.to_string(), 4),
        })
    }
    pub fn get(content: &str, id: i32) -> Result<Command<T>, ()> {
        match content {
            "ALL" => Ok(Command::GET {
                t: GetTypes::ALL,
                id,
            }),
            "FRD" => Ok(Command::GET {
                t: GetTypes::FRIENDS,
                id,
            }),
            "BLK" => Ok(Command::GET {
                t: GetTypes::BLOCKED,
                id,
            }),
            "PND" => Ok(Command::GET {
                t: GetTypes::PENDING,
                id,
            }),
            _ => {
                println!("{content}");
                Err(())
            }
        }
    }
    pub fn connect(content: &str, id: i32) -> Result<Command<T>, ()> {
        let other = content.parse().unwrap();

        Ok(Command::Connect { id, other })
    }
    pub fn block(content: &str, id: i32) -> Result<Command<T>, ()> {
        let other = content.parse().unwrap();

        Ok(Command::Block { id, other })
    }
    pub fn unblock(content: &str, id: i32) -> Result<Command<T>, ()> {
        let other = content.parse().unwrap();

        Ok(Command::Unblock { id, other })
    }
}
