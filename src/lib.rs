mod client;

mod container;

mod message;

mod commands;

use crate::{client::Client, commands::Command, container::Container};

use tokio::{io::AsyncBufReadExt, net::TcpStream, sync::mpsc};

pub async fn process(stream: TcpStream, tx: mpsc::Sender<Command>) {
    use Command::*;
    let (client, mut reader) = Client::new(stream).await;
    let name = client.name.clone();

    tx.send(Add { client }).await.unwrap();

    loop {
        println!("looping - {name}");
        let mut buf = String::new();
        let message = match reader.read_line(&mut buf).await {
            Ok(0) => {
                println!("recieved nothing");
                continue;
            }
            Ok(_) => {
                println!("got: {buf}");
                let x = buf.split_once(':');
                println!("after spliting: {x:?}");

                match x {
                    Some((n, m)) if !n.is_empty() => {
                        message::Message::new(name.clone(), n.to_owned(), m.to_owned())
                    }
                    _ => {
                        println!("error parsing message");
                        continue;
                    }
                }
            }
            Err(e) => {
                println!("{:?}", e);
                break;
            }
        };
        tx.send(Send { message }).await.unwrap();
    }

    // Remove the disconnected client from the list of clients.
    println!("{name} disconnected");
    tx.send(Remove { name }).await.unwrap();
}

pub async fn manager(mut rx: mpsc::Receiver<Command>) {
    let mut clients = Container::new();
    use Command::*;
    while let Some(command) = rx.recv().await {
        match command {
            Add { client } => {
                clients.push(client);
            }
            Send { message } => {
                let (name, message) = message.parse();
                let receiver = clients.get(&name);
                println!("sending '{message}' to {name}");
                receiver.send(&message).await;
                println!("message '{message}' sent to {name}");
            }
            Remove { name } => {
                clients.remove(&name);
            }
        };
        clients.print();
    }
}
