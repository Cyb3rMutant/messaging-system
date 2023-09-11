use crate::message::Message;
use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::TcpStream,
    time::{sleep, timeout},
};

#[derive(Debug)]
pub struct Client {
    pub name: String,
    reader: BufReader<ReadHalf<TcpStream>>,
    writer: WriteHalf<TcpStream>,
}

impl<'a> Client {
    pub async fn new(stream: TcpStream) -> Client {
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

        Client {
            name,
            reader,
            writer,
        }
    }

    pub async fn send(&mut self, message: &str) {
        println!("sending '{message}'");
        self.writer.write_all(message.as_bytes()).await.unwrap();
        println!("'{message}' sent");
    }

    async fn read(&mut self, buf: &'a mut String) -> tokio::io::Result<usize> {
        loop {
            println!("reading loop");
            match timeout(
                tokio::time::Duration::from_millis(500),
                self.reader.read_line(buf),
            )
            .await
            {
                Ok(len) => return len,
                _ => sleep(tokio::time::Duration::from_millis(100)).await,
            }
        }
    }

    pub async fn receive(&mut self, buf: &'a mut String) -> Option<Message> {
        println!("reading");
        match self.read(buf).await {
            Ok(s) if s > 0 => {
                println!("got: {buf}");
                let x = buf.split_once(':');
                println!("after spliting: {x:?}");

                match x {
                    Some((n, m)) if !n.is_empty() => {
                        Some(Message::new(self.name.clone(), n.to_owned(), m.to_owned()))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
