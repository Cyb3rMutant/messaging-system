mod client;

mod container;

mod message;

mod commands;

mod model;

pub mod manager;

use crate::commands::Command;

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
        let mut buf = String::new();
        // Prompt the client for their name.
        reader.read_line(&mut buf).await.unwrap();

        let (command, name_pass) = buf.split_once(';').unwrap();
        match command {
            "LGN" => {
                let (sender, rx) = oneshot::channel();
                tx.send(Add {
                    name_pass: name_pass.to_owned(),
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
            "REG" => {
                let (sender, rx) = oneshot::channel();
                tx.send(Register {
                    name_pass: name_pass.to_owned(),
                    sender,
                })
                .await
                .unwrap();
                match rx.await.unwrap() {
                    true => writer.write_all("REG;Y".as_bytes()).await.unwrap(),
                    false => writer.write_all("REG;N".as_bytes()).await.unwrap(),
                }
            }
            _ => writer
                .write_all("ERR;YOU ARE NOT LOGGED IN".as_bytes())
                .await
                .unwrap(),
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
