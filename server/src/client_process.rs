use std::sync::Arc;

use tokio::{
    io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf},
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use crate::{commands::Command, manager::Manager};

#[derive(Debug)]
pub struct Process {
    id: i32,
    reader: BufReader<ReadHalf<TcpStream>>,
    tx: mpsc::Sender<Command>,
    manager: Arc<Manager>,
}

impl Process {
    pub async fn run(stream: TcpStream, tx: mpsc::Sender<Command>, manager: Arc<Manager>) {
        use Command::*;
        let (reader, mut writer) = split(stream);
        let reader = BufReader::new(reader);
        let mut p = Process {
            id: 0,
            reader,
            tx,
            manager,
        };

        p.id = loop {
            let mut buf = String::new();
            println!("2 loop - {:?}", buf);
            match p.reader.read_line(&mut buf).await {
                Ok(0) => {
                    println!("recieved nothing");
                    return;
                }
                Err(e) => {
                    println!("{:?}", e);
                    return;
                }
                _ => (),
            }
            println!("2 loop - {:?}", buf);

            let (command, name_pass) = buf.split_once(';').unwrap();
            println!("{:?}{name_pass}", command);
            match command {
                "TESTINGCLEAR" => {
                    println!("in clear");
                    p.tx.send(Command::Testing_Clear).await.unwrap();
                }
                "LGN" => {
                    println!("3 login");
                    let (sender, rx) = oneshot::channel();
                    p.tx.send(Add {
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
                    p.tx.send(Register {
                        name_pass: name_pass.to_owned(),
                        sender,
                    })
                    .await
                    .unwrap();
                    println!("sent");
                    match rx.await.unwrap() {
                        true => writer.write_all("REG;Y\n".as_bytes()).await.unwrap(),
                        false => writer.write_all("REG;N\n".as_bytes()).await.unwrap(),
                    }
                    println!("done");
                }
                _ => writer
                    .write_all("ERR;YOU ARE NOT LOGGED IN\n".as_bytes())
                    .await
                    .unwrap(),
            }
            println!("7 end loop");
        };

        p.main().await;
    }
    async fn main(mut self) {
        loop {
            println!("looping - {}", self.id);
            let message = self.get_command().await;

            println!("{message:?}");

            match message {
                Ok(message) => self.tx.send(message).await.unwrap(),
                Err(e) => {
                    println!("parse error {e:?}");
                    break;
                }
            }
        }

        // Remove the disconnected client from the list of clients.
        println!("{} disconnected", self.id);
        self.tx.send(Command::Remove { id: self.id }).await.unwrap();
    }
    async fn get_command(&mut self) -> Result<Command, ()> {
        let mut buf = String::new();
        match self.reader.read_line(&mut buf).await {
            Ok(0) => {
                println!("recieved nothing");
                return Err(());
            }
            Err(e) => {
                println!("{:?}", e);
                return Err(());
            }
            _ => (),
        }
        let Some((command, content)) = buf.trim().split_once(';') else {
                return Err(());
                    };
        println!("{command} {content}");
        match command {
            "TESTINGCLEAR" => {
                println!("in clear");
                Ok(Command::Testing_Clear)
            }
            "SND" => Command::send(content, self.id),
            "CNT" => Command::connect(content, self.id),
            "GET" => Command::get(content, self.id),
            "STS" => Command::status(content, self.id),
            "DEL" => Command::delete(content, self.id),
            "UPD" => Command::update(content, self.id),
            "A" => Command::a(content, self.id),

            _ => Err(()),
        }
    }
}
