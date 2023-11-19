use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Client {
    pub name: String,
    writer: WriteHalf<TcpStream>,
    pub friends: Vec<String>,
}

impl<'a> Client {
    pub async fn new(stream: TcpStream) -> (Client, BufReader<ReadHalf<TcpStream>>) {
        let mut name = String::new();
        let (reader, mut writer) = split(stream);

        let mut reader = BufReader::new(reader);

        // Prompt the client for their name.
        reader.read_line(&mut name).await.unwrap();

        let name = name.trim().to_string();

        let message = format!("LGN;{}!\n", name);
        writer.write_all(message.as_bytes()).await.unwrap();

        (
            Client {
                name,
                writer,
                friends: vec![],
            },
            reader,
        )
    }

    pub async fn send(&mut self, message: &str) {
        self.writer.write_all(message.as_bytes()).await.unwrap();
    }

    pub fn add_friend(&mut self, name: &str) {
        self.friends.push(name.to_owned());
    }
}
