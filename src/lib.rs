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
                let Some((command, content)) = buf.split_once(';') else {
                        break;
                    };
                match command {
                    "SND" => Command::send(content, name.clone()),
                    "CNT" => Command::connect(content, name.clone()),
                    "GET" => Ok(Command::GET { name: name.clone() }),
                    _ => break,
                }
            }
            Err(e) => {
                println!("{:?}", e);
                break;
            }
        };

        match message {
            Ok(message) => tx.send(message).await.unwrap(),
            _ => {
                println!("parse error");
                break;
            }
        }
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
                receiver.send(&message).await;
            }
            Connect { me, other } => {
                {
                    let me = clients.get(&me);
                    me.add_friend(&other);
                }
                {
                    let other = clients.get(&other);
                    other.add_friend(&me);
                }
            }
            GET { name } => {
                let names = clients.get_all();
                clients.get(&name).send(&names).await;
            }
            Remove { name } => {
                clients.remove(&name);
            }
        };
        clients.print();
    }
}
