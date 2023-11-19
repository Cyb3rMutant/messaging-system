use crate::client::Client;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Container {
    clients: HashMap<String, Client>,
}

impl Container {
    pub fn new() -> Self {
        Container {
            clients: HashMap::new(),
        }
    }

    pub fn push(&mut self, client: Client) {
        self.clients.insert(client.name.clone(), client);
    }

    pub fn remove(&mut self, name: &str) {
        self.clients.remove(name).unwrap();
    }

    pub fn get(&mut self, name: &str) -> &mut Client {
        self.clients.get_mut(name).unwrap()
    }

    pub fn print(&self) {
        for c in self.clients.keys() {
            println!("{:?}", c);
        }
    }

    pub fn get_all(&self) -> String {
        let mut list = String::new();

        for c in self.clients.keys() {
            list.push(';');
            list += c;
        }

        list
    }
}
