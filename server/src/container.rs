use tokio::{io::WriteHalf, net::TcpStream};

use crate::{client::Client, message::Message};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Container {
    clients: HashMap<String, Option<Client>>,
}

impl Container {
    pub fn new(users: Vec<String>) -> Self {
        Container {
            clients: users.into_iter().map(|u| (u, None)).collect(),
        }
    }

    fn push(&mut self, client: Client) {
        let c = self.clients.get_mut(&client.name).unwrap();
        *c = Some(client);
    }

    pub async fn login(
        &mut self,
        name: &str,
        writer: WriteHalf<TcpStream>,
        messages: Vec<Message>,
    ) {
        // ) -> Result<(), WriteHalf<TcpStream>> {
        // add the functionality to not login if the user has already logged in
        // if self.clients.contains_key(name) {
        //     return Err(writer);
        // }
        let client = Client::new(name.to_owned(), writer).await;
        let messages = serde_json::to_string(&messages).unwrap();
        let message = format!("LGN;{};{}\n", name, messages);
        self.push(client);
        self.send(name, &message).await;
        // Ok(())
    }

    pub fn remove(&mut self, name: &str) {
        *self.clients.get_mut(name).unwrap() = None;
    }

    #[allow(dead_code)]
    pub fn print(&self) {
        for c in self.clients.keys() {
            println!("{:?}", self.clients.get(c));
            println!("{:?}", c);
        }
    }

    pub fn get_all(&self) -> String {
        let mut list = String::new();

        for c in self.clients.keys() {
            list.push(';');
            list += c;
        }
        println!("{list:?}");
        self.print();

        list
    }

    pub async fn send_users(&mut self, name: &str) {
        self.send(name, &format!("USR{}\n", self.get_all())).await;
    }

    pub fn add_friends(&mut self, me: &str, other: &str) {
        if let Some(me) = self.clients.get_mut(me).unwrap().as_mut() {
            me.add_friend(&other);
        };
        if let Some(other) = self.clients.get_mut(other).unwrap().as_mut() {
            other.add_friend(&me);
        };
    }

    pub async fn send(&mut self, name: &str, message: &str) {
        let x = self.clients.get_mut(name).unwrap().as_mut();
        if let Some(c) = x {
            c.send(message).await;
        }
    }
}
