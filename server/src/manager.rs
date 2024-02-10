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
        let pool = sqlx::mysql::MySqlPool::connect("mysql://yazeed@localhost:3306/messaging")
            .await
            .unwrap();

        let clients = Container::new(model::load_users(&pool).await);
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
                    println!("start manager");
                    let (name, password) = name_pass.trim().split_once(';').unwrap();
                    println!("{:?}{:?}\n", name, password);
                    if let Ok(_) = model::login(&name, &password, &self.pool).await {
                        println!("in\n");
                        let mut clients = self.clients.lock().await;
                        let messages = model::load_messages(name, &self.pool).await;
                        clients.login(name, writer, messages).await;
                        sender.send(Ok(name.to_owned())).unwrap();
                    } else {
                        println!("wrong\n");
                        let message = format!("ERR;PWD\n");
                        writer.write_all(message.as_bytes()).await.unwrap();
                        sender.send(Err(writer)).unwrap();
                    }
                }
                Register { name_pass, sender } => {
                    // might want to add the registered user to the container, as its only done in
                    // the database
                    // that way you could even do the authentication lacally rather than having to
                    // check db
                    let (name, password) = name_pass.trim().split_once(';').unwrap();
                    println!("{:?}{:?}\n", name, password);
                    if let Ok(_) = model::register(&name, &password, &self.pool).await {
                        sender.send(true).unwrap();
                    } else {
                        sender.send(false).unwrap();
                    };
                }
                Send { message } => {
                    model::new_message(&message, &self.pool).await.unwrap();
                    let (name, message) = message.parse();
                    println!("{name:?} {message:?}");
                    let mut clients = self.clients.lock().await;
                    clients.send(&name, &message).await;
                }
                Connect { me, other } => {
                    let mut clients = self.clients.lock().await;
                    clients.add_friends(&me, &other);
                }
                GET { name } => {
                    let mut clients = self.clients.lock().await;
                    clients.send_users(&name).await;
                    println!("done");
                }
                Remove { name } => {
                    let mut clients = self.clients.lock().await;
                    clients.remove(&name);
                }
            };
        }
    }
}
