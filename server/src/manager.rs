// should try to change this back to std mutex
use tokio::sync::Mutex;

use sqlx::{MySql, Pool};
use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{commands::Command, container::Container, model};

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
                    if let Ok(id) = model::login(&name, &password, &self.pool).await {
                        println!("in\n");
                        let mut clients = self.clients.lock().await;
                        let messages = model::load_messages(id, &self.pool).await;
                        clients.login(id, writer, messages).await;
                        println!("{id}");
                        sender.send(Ok(id)).unwrap();
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
                    let m = format!("MSG;{};{}", message.chat_id, message.content);
                    println!("{message:?}");
                    let mut clients = self.clients.lock().await;
                    let receiver = clients.get_other(message.chat_id, message.sender_id);
                    clients.send(receiver, &m).await;
                }
                Connect { me, other } => {
                    let mut clients = self.clients.lock().await;
                    clients.add_friends(&me, &other);
                }
                GET { id } => {
                    let mut clients = self.clients.lock().await;
                    clients.send_users(id).await;
                    println!("done");
                }
                Remove { id } => {
                    let mut clients = self.clients.lock().await;
                    clients.remove(id);
                }
            };
        }
    }
}
