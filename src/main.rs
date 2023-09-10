use std::collections::HashMap;
use std::usize;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, timeout};

#[derive(Debug)]
enum Command {
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

#[derive(Debug)]
struct Client {
    name: String,
    reader: BufReader<ReadHalf<TcpStream>>,
    writer: WriteHalf<TcpStream>,
}

impl<'a> Client {
    async fn new(stream: TcpStream) -> Client {
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

    async fn send(&mut self, message: &str) {
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

    async fn receive(&mut self, buf: &'a mut String) -> Option<Message> {
        println!("reading");
        match self.read(buf).await {
            Ok(s) if s > 0 => {
                println!("got: {buf}");
                let x = buf.split_once(':');
                println!("after spliting: {x:?}");

                match x {
                    Some((n, m)) if !n.is_empty() => Some(Message {
                        sender: self.name.clone(),
                        receiver: n.to_owned(),
                        content: m.to_owned(),
                    }),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct Message {
    sender: String,
    receiver: String,
    content: String,
}
impl Message {
    fn parse(self) -> (String, String) {
        (self.receiver, format!("{}: {}", self.sender, self.content))
    }
}

#[derive(Debug)]
struct Container {
    clients: HashMap<String, Client>,
}

impl Container {
    fn new() -> Self {
        Container {
            clients: HashMap::new(),
        }
    }

    fn push(&mut self, client: Client) {
        self.clients.insert(client.name.clone(), client);
    }
}

async fn handle_client(stream: TcpStream, tx: mpsc::Sender<Command>) {
    use Command::*;
    let (sender, rx) = oneshot::channel();
    tx.send(Add { stream, sender }).await.unwrap();
    let name = rx.await.unwrap();

    loop {
        println!("looping - {name}");
        let (sender, rx) = oneshot::channel();
        tx.send(Receive {
            name: name.clone(),
            sender,
        })
        .await
        .unwrap();
        match rx.await.unwrap() {
            Some(m) => {
                // Send messages to all clients.
                let (name, message) = m.parse();
                tx.send(Send { name, message }).await.unwrap();
            }
            _ => break,
        }
    }

    // Remove the disconnected client from the list of clients.
    println!("{name} disconnected");
    tx.send(Remove { name }).await.unwrap();
}

async fn manager(mut clients: Container, mut rx: mpsc::Receiver<Command>) {
    use Command::*;
    while let Some(command) = rx.recv().await {
        match command {
            Add { stream, sender } => {
                let client = Client::new(stream).await;
                let client_name = client.name.clone();
                clients.push(client);
                sender.send(client_name).unwrap();
            }
            Receive { name, sender } => {
                let mut buf = String::new();
                let data = clients
                    .clients
                    .get_mut(&name)
                    .unwrap()
                    .receive(&mut buf)
                    .await;

                sender.send(data).unwrap();
            }
            Send { name, message } => {
                let receiver = clients.clients.get_mut(&name).unwrap();
                receiver.send(&message).await;
            }
            Remove { name } => {
                clients.clients.remove(&name).unwrap();
            }
        };
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server listening on 127.0.0.1:8080");

    let (tx, rx) = mpsc::channel(32);

    tokio::spawn(async move {
        let clients = Container::new();
        manager(clients, rx).await;
    });

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("{stream:?} {addr:?}");
        let tx = tx.clone();

        tokio::spawn(async move {
            handle_client(stream, tx).await;
        });
    }
}
