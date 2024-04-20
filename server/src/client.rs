use tokio::io::{AsyncWriteExt, WriteHalf};

pub struct Client<T: AsyncWriteExt> {
    pub id: i32,
    pub name: String,
    writer: Option<WriteHalf<T>>,
}

impl<T: AsyncWriteExt> Client<T> {
    pub fn new(id: i32, name: String) -> Self {
        Client {
            id,
            name,
            writer: None,
        }
    }
    pub fn login(&mut self, writer: WriteHalf<T>) -> Result<(), WriteHalf<T>> {
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
