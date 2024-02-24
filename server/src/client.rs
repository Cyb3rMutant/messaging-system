use tokio::{
    io::{AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

#[derive(Debug)]
pub struct Client {
    pub id: i32,
    pub name: String,
    writer: Option<WriteHalf<TcpStream>>,
}

impl Client {
    pub fn new(id: i32, name: String) -> Client {
        Client {
            id,
            name,
            writer: None,
        }
    }
    pub fn login(&mut self, writer: WriteHalf<TcpStream>) -> Result<(), WriteHalf<TcpStream>> {
        if self.writer.is_some() {
            Err(writer)
        } else {
            self.writer = Some(writer);
            Ok(())
        }
    }

    pub fn loguot(&mut self) {
        self.writer = None;
    }

    pub async fn send(&mut self, message: &str) {
        println!("sending {:?}", message);
        if let Some(w) = self.writer.as_mut() {
            w.write_all(message.as_bytes()).await.unwrap();
        }
        println!("done 1");
    }
}
