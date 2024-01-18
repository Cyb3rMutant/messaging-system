// should try to change this back to std mutex
use tokio::sync::Mutex;

use sqlx::{MySql, Pool};
use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{client::Client, commands::Command, container::Container, model};

#[derive(Debug)]
pub struct Manager {
    rx: Mutex<mpsc::Receiver<Command>>,
    clients: Mutex<Container>,
    pool: Pool<MySql>,
}

impl Manager {
    pub async fn new(rx: mpsc::Receiver<Command>) -> Manager {
        let pool = sqlx::mysql::MySqlPool::connect("mysql://yazeed@localhost:3306/rustdb")
            .await
            .unwrap();

        let clients = Container::new();
        Manager {
            rx: Mutex::new(rx),
            clients: Mutex::new(clients),
            pool,
        }
    }
    pub async fn run(&self) {
        use Command::*;

        while let Some(command) = self.rx.lock().await.recv().await {
            match command {
                Add {
                    name_pass,
                    mut writer,
                    sender,
                } => {
                    let (name, password) = name_pass.trim().split_once(';').unwrap();
                    println!("{:?}{:?}\n", name, password);
                    if let Ok(_) = model::login(&name, &password, &self.pool).await {
                        println!("in\n");
                        let name = name.to_owned();
                        let mut clients = self.clients.lock().await;
                        clients.push(Client::new(name.clone(), writer).await);
                        sender.send((Some(name), None)).unwrap();
                    } else {
                        println!("wrong\n");
                        let message = format!("ERR;PWD!\n");
                        writer.write_all(message.as_bytes()).await.unwrap();
                        sender.send((None, Some(writer))).unwrap();
                    }
                }
                Register { name_pass, sender } => {
                    let (name, password) = name_pass.trim().split_once(';').unwrap();
                    println!("{:?}{:?}\n", name, password);
                    if let Ok(_) = model::register(&name, &password, &self.pool).await {
                        sender.send(true).unwrap();
                    } else {
                        sender.send(false).unwrap();
                    };
                }
                Send { message } => {
                    let (name, message) = message.parse();
                    println!("{name:?} {message:?}");
                    let mut clients = self.clients.lock().await;
                    let receiver = clients.get(&name);
                    receiver.send(&message).await;
                }
                Connect { me, other } => {
                    let mut clients = self.clients.lock().await;
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
                    let mut clients = self.clients.lock().await;
                    let names = clients.get_all();
                    let names = format!("USR{names}\n");
                    clients.get(&name).send(&names).await;
                }
                Remove { name } => {
                    let mut clients = self.clients.lock().await;
                    clients.remove(&name);
                }
            };
        }
    }
}
