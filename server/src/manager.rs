// should try to change this back to std mutex
use tokio::sync::Mutex;

use sqlx::{MySql, Pool};
use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{
    commands::{Command, GetTypes},
    container::Container,
    model,
};

#[derive(Debug)]
pub struct Manager {
    rx: Mutex<mpsc::Receiver<Command>>,
    clients: Mutex<Container>,
    pool: Pool<MySql>,
    testing: bool,
}

impl Manager {
    pub async fn new(rx: mpsc::Receiver<Command>) -> Manager {
        let db = "messaging";
        let pool =
            sqlx::mysql::MySqlPool::connect(format!("mysql://yazeed@localhost:3306/{db}").as_str())
                .await
                .unwrap();

        let clients = Container::new(
            model::load_chats(&pool).await,
            model::load_lonely(&pool).await,
        );
        Manager {
            rx: Mutex::new(rx),
            clients: Mutex::new(clients),
            pool,
            testing: if db == "testing" { true } else { false },
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
                        match clients.login(id, writer, messages).await {
                            Ok(_) => {
                                println!("{id}");
                                sender.send(Ok(id)).unwrap();
                            }
                            Err(w) => sender.send(Err(w)).unwrap(),
                        };
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
                    if let Ok(id) = model::register(&name, &password, &self.pool).await {
                        let mut clients = self.clients.lock().await;
                        clients.new_user(id, name.to_owned());
                        sender.send(true).unwrap();
                    } else {
                        sender.send(false).unwrap();
                    };
                }
                Send { mut message } => {
                    let mut clients = self.clients.lock().await;
                    message.message_id = model::new_message(&message, &self.pool).await;
                    clients
                        .send(message.sender_id, &format!("MID;{}\n", message.message_id))
                        .await;

                    let m = format!(
                        "MSG;{};{};{}\n",
                        message.chat_id, message.message_id, message.content
                    );
                    println!("{message:?}");
                    let receiver = clients.get_other(message.chat_id, message.sender_id);
                    clients.send(receiver, &m).await;
                }
                Connect { id, other } => {
                    let mut clients = self.clients.lock().await;
                    let chat_id = model::connect(id, other, &self.pool).await;
                    clients.add_friends(id, other, chat_id);
                    clients.send(id, &format!("CNT;{chat_id};{other}\n")).await;
                    clients.send(other, &format!("CNT;{chat_id};{id}\n")).await;
                }
                GET { t, id } => {
                    let mut clients = self.clients.lock().await;
                    use GetTypes::*;
                    match t {
                        ALL => clients.send_all(id).await,
                        // PENDING => clients.send_pending(id).await,
                        FRIENDS => clients.send_friends(id).await,
                        // BLOCKED => clients.send_blocked(id).await,
                        _ => (),
                    }
                }
                Remove { id } => {
                    let mut clients = self.clients.lock().await;
                    clients.remove(id);
                }
                Status { chat_id, id } => {
                    let mut clients = self.clients.lock().await;
                    let receiver = clients.get_other(chat_id, id);
                    model::set_seen(chat_id, receiver, &self.pool).await;
                    clients.send(receiver, &format!("STS;{chat_id}\n")).await;
                }
                Delete {
                    chat_id,
                    id,
                    message_id,
                } => {
                    let mut clients = self.clients.lock().await;
                    let receiver = clients.get_other(chat_id, id);
                    model::delete(message_id, &self.pool).await;
                    clients
                        .send(receiver, &format!("DEL;{chat_id};{message_id}\n"))
                        .await;
                }
                Update { message } => {
                    let mut clients = self.clients.lock().await;
                    let receiver = clients.get_other(message.chat_id, message.sender_id);
                    model::update(&message, &self.pool).await;
                    clients
                        .send(
                            receiver,
                            &format!(
                                "UPD;{};{};{}\n",
                                message.chat_id, message.message_id, message.content,
                            ),
                        )
                        .await;
                }
                Testing_Clear => {
                    if self.testing {
                        model::clear(&self.pool).await;
                    }
                }
            };
        }
    }
}
