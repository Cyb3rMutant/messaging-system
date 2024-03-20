use tokio::{io::WriteHalf, net::TcpStream, sync::oneshot};

use crate::message::Message;

#[derive(Debug)]
pub enum Command {
    Add {
        name_pass: String,
        writer: WriteHalf<TcpStream>,
        sender: oneshot::Sender<Result<i32, WriteHalf<TcpStream>>>,
    },
    Register {
        name_pass: String,
        sender: oneshot::Sender<bool>,
    },
    Connect {
        me: String,
        other: String,
    },
    Send {
        message: Message,
    },
    Remove {
        id: i32,
    },
    GET {
        id: i32,
    },
    UPDATE {
        chat_id: i32,
        id: i32,
        new_status: i32,
    },
}

impl Command {
    pub fn send(content: &str, id: i32) -> Result<Command, ()> {
        match content.split_once(';') {
            Some((chat_id, message)) => Ok(Command::Send {
                message: Message::new(chat_id.parse().unwrap(), id, message.to_owned(), 1),
            }),
            _ => Err(()),
        }
    }

    // pub fn connect(other: &str, name: String) -> Result<Command, String> {
    //     let other = other.trim_end_matches("\r\n").to_owned();
    //
    //     Ok(Command::Connect {
    //         me: name,
    //         other: other.to_owned(),
    //     })
    // }
}
