use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Client {
    pub name: String,
    writer: WriteHalf<TcpStream>,
    pub friends: Vec<String>,
}

impl<'a> Client {
    pub async fn new(name: String, mut writer: WriteHalf<TcpStream>) -> Client {
        let message = format!("LGN;{}!\n", name);
        writer.write_all(message.as_bytes()).await.unwrap();
        Client {
            name,
            writer,
            friends: vec![],
        }
    }

    pub async fn send(&mut self, message: &str) {
        self.writer.write_all(message.as_bytes()).await.unwrap();
    }

    pub fn add_friend(&mut self, name: &str) {
        self.friends.push(name.to_owned());
    }
}
