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
    println!("1 start");

    let (reader, mut writer) = split(stream);

    let mut reader = BufReader::new(reader);
    //////////////////////////
    let id = loop {
        let mut buf = String::new();
        println!("2 loop - {:?}", buf);
        reader.read_line(&mut buf).await.unwrap();
        println!("2 loop - {:?}", buf);

        let (command, name_pass) = buf.split_once(';').unwrap();
        println!("{:?}{name_pass}", command);
        match command {
            "LGN" => {
                println!("3 login");
                let (sender, rx) = oneshot::channel();
                tx.send(Add {
                    name_pass: name_pass.to_owned(),
                    writer,
                    sender,
                })
                .await
                .unwrap();
                println!("4 sent to manager");
                match rx.await.unwrap() {
                    Ok(id) => {
                        println!("5 logged in");
                        break id;
                    }
                    Err(w) => {
                        println!("6 wrong");
                        writer = w;
                    }
                }
            }
            "REG" => {
                println!("in reg");
                let (sender, rx) = oneshot::channel();
                tx.send(Register {
                    name_pass: name_pass.to_owned(),
                    sender,
                })
                .await
                .unwrap();
                println!("sent");
                match rx.await.unwrap() {
                    true => writer.write_all("REG;Y".as_bytes()).await.unwrap(),
                    false => writer.write_all("REG;N".as_bytes()).await.unwrap(),
                }
                println!("done");
            }
            _ => writer
                .write_all("ERR;YOU ARE NOT LOGGED IN".as_bytes())
                .await
                .unwrap(),
        }
        println!("7 end loop");
    };
    //////////////////////////
    println!("8 done loop 1");

    loop {
        println!("looping - {id}");
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
                    "SND" => Command::send(content, id),
                    // "CNT" => Command::connect(content, id),
                    "GET" => Ok(Command::GET { id }),
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
    println!("{id} disconnected");
    tx.send(Remove { id }).await.unwrap();
}
