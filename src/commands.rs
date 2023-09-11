use tokio::{net::TcpStream, sync::oneshot};

use crate::message::Message;

#[derive(Debug)]
pub enum Command {
    Add {
        stream: TcpStream,
        sender: oneshot::Sender<String>,
    },
    Receive {
        name: String,
        sender: oneshot::Sender<Option<Message>>,
    },
    Send {
        name: String,
        message: String,
    },
    Remove {
        name: String,
    },
}
