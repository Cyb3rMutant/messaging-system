mod client;

mod container;

mod message;

mod commands;

mod model;

use crate::{client::Client, commands::Command, container::Container};

use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

pub async fn process(stream: TcpStream, tx: mpsc::Sender<Command>) {
    use Command::*;
    // let (client, mut reader) = Client::new(stream).await;
    // let name = client.name.clone();

    let (reader, mut writer) = split(stream);

    let mut reader = BufReader::new(reader);
    //////////////////////////
    let name = loop {
        let mut name_pass = String::new();
        // Prompt the client for their name.
        reader.read_line(&mut name_pass).await.unwrap();
        let (sender, rx) = oneshot::channel();

        tx.send(Add {
            name_pass,
            writer,
            sender,
        })
        .await
        .unwrap();
        match rx.await.unwrap() {
            (Some(name), None) => {
                break name;
            }
            (None, Some(w)) => {
                writer = w;
            }
            _ => {
                panic!();
            }
        }
    }
    .to_owned();
    //////////////////////////

    loop {
        println!("looping - {name}");
        let mut buf = String::new();
        let message = match reader.read_line(&mut buf).await {
            Ok(0) => {
                println!("recieved nothing");
                break;
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
        println!("{message:?}");

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
    let pool = sqlx::mysql::MySqlPool::connect("mysql://yazeed@localhost:3306/rustdb")
        .await
        .unwrap();

    let mut clients = Container::new();

    use Command::*;

    while let Some(command) = rx.recv().await {
        match command {
            Add {
                name_pass,
                mut writer,
                sender,
            } => {
                let (name, password) = name_pass.trim().split_once(';').unwrap();
                println!("{:?}{:?}\n", name, password);
                if let Ok(_) = model::login(&name, &password, &pool).await {
                    println!("in\n");
                    let name = name.to_owned();
                    clients.push(Client::new(name.clone(), writer).await);
                    sender.send((Some(name), None)).unwrap();
                } else {
                    println!("wrong\n");
                    let message = format!("ERR;PWD!\n");
                    writer.write_all(message.as_bytes()).await.unwrap();
                    sender.send((None, Some(writer))).unwrap();
                }
            }
            Send { message } => {
                let (name, message) = message.parse();
                println!("{name:?} {message:?}");
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
                let names = format!("USR{names}\n");
                clients.get(&name).send(&names).await;
            }
            Remove { name } => {
                clients.remove(&name);
            }
        };
        clients.print();
    }
}
