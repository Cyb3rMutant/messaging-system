// should try to change this back to std mutex
use tokio::sync::Mutex;

use tokio::{io::AsyncWriteExt, sync::mpsc};

use crate::{
    commands::{Command, GetTypes},
    container::Container,
    model::Model,
};

pub struct Manager<T: AsyncWriteExt> {
    rx: mpsc::Receiver<Command<T>>,
    clients: Mutex<Container<T>>,
    model: Model,
    testing: bool,
}

impl<T: AsyncWriteExt> Manager<T> {
    pub async fn new(rx: mpsc::Receiver<Command<T>>) -> Self {
        let db = "messaging";
        let model = Model::new(db).await;

        let clients = Container::new(model.load_chats().await, model.load_lonely().await);
        Manager {
            rx,
            clients: Mutex::new(clients),
            model,
            testing: if db == "testing" { true } else { false },
        }
    }
    pub async fn run(&mut self) {
        use Command::*;

        while let Some(command) = self.rx.recv().await {
            match command {
                Add {
                    name_pass,
                    mut writer,
                    sender,
                } => {
                    println!("start manager");
                    let (name, password) = name_pass.trim().split_once(';').unwrap();
                    println!("{:?}{:?}\n", name, password);
                    if let Ok(id) = self.model.login(&name, &password).await {
                        println!("in\n");
                        let mut clients = self.clients.lock().await;
                        let messages = self.model.load_messages(id).await;
                        let p_g = self.model.chats_p_g_B(id).await;
                        match clients.login(id, writer, messages, p_g).await {
                            Ok(_) => {
                                println!("{id}");
                                let _ = sender.send(Ok(id));
                            }
                            Err(w) => {
                                let _ = sender.send(Err(w));
                            }
                        };
                    } else {
                        println!("wrong\n");
                        let message = format!("ERR;PWD\n");
                        writer.write_all(message.as_bytes()).await.unwrap();
                        let _ = sender.send(Err(writer));
                    }
                }
                Register { name_pass, sender } => {
                    // might want to add the registered user to the container, as its only done in
                    // the database
                    // that way you could even do the authentication lacally rather than having to
                    // check db
                    let (name, password) = name_pass.trim().split_once(';').unwrap();
                    println!("{:?}{:?}\n", name, password);
                    if let Ok(id) = self.model.register(&name, &password).await {
                        let mut clients = self.clients.lock().await;
                        clients.new_user(id, name.to_owned());
                        sender.send(true).unwrap();
                    } else {
                        sender.send(false).unwrap();
                    };
                }
                Send { mut message } => {
                    let mut clients = self.clients.lock().await;
                    message.message_id = self.model.new_message(&message).await;
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
                    let (p, g) = self.model.connect(id, other).await;
                    clients.add_friends(id, other, p);
                    clients.send(id, &format!("CNT;{p};{other};{g}\n")).await;
                    clients.send(other, &format!("CNT;{p};{id};{g}\n")).await;
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
                    self.model.set_seen(chat_id, receiver).await;
                    clients.send(receiver, &format!("STS;{chat_id}\n")).await;
                }
                Delete {
                    chat_id,
                    id,
                    message_id,
                } => {
                    let mut clients = self.clients.lock().await;
                    let receiver = clients.get_other(chat_id, id);
                    self.model.delete(message_id).await;
                    clients
                        .send(receiver, &format!("DEL;{chat_id};{message_id}\n"))
                        .await;
                }
                Update { message } => {
                    let mut clients = self.clients.lock().await;
                    let receiver = clients.get_other(message.chat_id, message.sender_id);
                    self.model.update(&message).await;
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
                A { chat_id, id, A } => {
                    let mut clients = self.clients.lock().await;
                    let receiver = clients.get_other(chat_id, id);
                    self.model.set_ab(chat_id, id, A).await;
                    clients.send(receiver, &format!("B;{chat_id};{A}\n")).await;
                }
                Testing_Clear => {
                    if self.testing {
                        self.model.clear().await;
                    }
                }
            };
        }
    }
}
