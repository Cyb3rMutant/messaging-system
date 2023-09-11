mod client;

pub mod container;

mod message;

mod commands;

use crate::{client::Client, commands::Command, container::Container};

use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

pub async fn process(stream: TcpStream, tx: mpsc::Sender<Command>) {
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

pub async fn manager(mut clients: Container, mut rx: mpsc::Receiver<Command>) {
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
                let data = clients.get(&name).receive(&mut buf).await;

                sender.send(data).unwrap();
            }
            Send { name, message } => {
                let receiver = clients.get(&name);
                receiver.send(&message).await;
            }
            Remove { name } => {
                clients.remove(&name);
            }
        };
    }
}
