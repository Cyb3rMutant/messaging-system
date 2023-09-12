use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Client {
    pub name: String,
    writer: WriteHalf<TcpStream>,
}

impl<'a> Client {
    pub async fn new(stream: TcpStream) -> (Client, BufReader<ReadHalf<TcpStream>>) {
        let mut name = String::new();
        let (reader, mut writer) = split(stream);

        let mut reader = BufReader::new(reader);

        // Prompt the client for their name.
        println!("writing for name");
        writer.write_all(b"Enter your name: ").await.unwrap();
        println!("reading name");
        reader.read_line(&mut name).await.unwrap();
        let name = name.trim().to_string();

        println!("{name} joined");
        let message = format!("Welcome, {}!\n", name);
        writer.write_all(message.as_bytes()).await.unwrap();

        (Client { name, writer }, reader)
    }

    pub async fn send(&mut self, message: &str) {
        println!("sending '{message}'");
        self.writer.write_all(message.as_bytes()).await.unwrap();
        println!("'{message}' sent");
    }
}
