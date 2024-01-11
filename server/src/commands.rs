use tokio::{io::WriteHalf, net::TcpStream, sync::oneshot};

use crate::message::Message;

#[derive(Debug)]
pub enum Command {
    Add {
        name_pass: String,
        writer: WriteHalf<TcpStream>,
        sender: oneshot::Sender<(Option<String>, Option<WriteHalf<TcpStream>>)>,
    },
    Connect {
        me: String,
        other: String,
    },
    Send {
        message: Message,
    },
    Remove {
        name: String,
    },
    GET {
        name: String,
    },
}

impl Command {
    pub fn send(content: &str, name: String) -> Result<Command, String> {
        let x = content.split_once(';');

        match x {
            Some((n, m)) if !n.is_empty() => Ok(Command::Send {
                message: Message::new(name, n.to_owned(), m.to_owned()),
            }),
            _ => Err(String::new()),
        }
    }

    pub fn connect(other: &str, name: String) -> Result<Command, String> {
        let other = other.trim_end_matches("\r\n").to_owned();

        Ok(Command::Connect {
            me: name,
            other: other.to_owned(),
        })
    }
}
