use petgraph::{graph::NodeIndex, graph::UnGraph, Graph, Undirected};
use tokio::{io::WriteHalf, net::TcpStream};

use crate::{client::Client, message::Message};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Container {
    nodes: HashMap<String, NodeIndex>,
    network: Graph<Client, i32, Undirected>,
}

impl Container {
    pub fn new(users: Vec<(String, String, i32)>) -> Self {
        let mut nodes = HashMap::new();
        let mut network = UnGraph::new_undirected();

        for (username_1, username_2, chat_id) in users {
            if !nodes.contains_key(&username_1) {
                let n = network.add_node(Client::new(username_1.clone()));
                nodes.insert(username_1.clone(), n);
            }
            if !nodes.contains_key(&username_2) {
                let n = network.add_node(Client::new(username_2.clone()));
                nodes.insert(username_2.clone(), n);
            }
            let node_1 = nodes.get(&username_1).unwrap();
            let node_2 = nodes.get(&username_2).unwrap();
            network.add_edge(*node_1, *node_2, chat_id);
        }
        // clients: users.into_iter().map(|u| (u, None)).collect(),
        Container { nodes, network }
    }

    // fn push(&mut self, client: Client) {
    //     let c = self.clients.get_mut(&client.name).unwrap();
    //     *c = Some(client);
    // }

    pub async fn login(
        &mut self,
        name: &str,
        writer: WriteHalf<TcpStream>,
        messages: Vec<Message>,
    ) {
        let messages = serde_json::to_string(&messages).unwrap();
        let message = format!("LGN;{};{}\n", name, messages);
        let node = *self.nodes.get(name).unwrap();
        self.network[node].login(writer);
        self.send(name, &message).await;
    }

    pub fn remove(&mut self, name: &str) {
        let node = *self.nodes.get(name).unwrap();
        self.network[node].loguot();
    }

    pub fn get_all(&self) -> String {
        let mut list = String::new();

        for c in self.nodes.keys() {
            list.push(';');
            list += c;
        }
        println!("{list:?}");

        list
    }
    fn get_friends(&self, name: &str) -> String {
        let mut list = String::new();

        let node = *self.nodes.get(name).unwrap();
        for u in self.network.neighbors(node) {
            list.push(';');
            list += &self.network[u].name;
        }

        println!("{list:?}");
        list
    }

    pub async fn send_users(&mut self, name: &str) {
        // self.send(name, &format!("USR{}\n", self.get_all())).await;
        self.send(name, &format!("USR{}\n", self.get_friends(name)))
            .await;
    }

    pub fn add_friends(&mut self, me: &str, other: &str) {
        unimplemented!()
        // if let Some(me) = self.clients.get_mut(me).unwrap().as_mut() {
        //     me.add_friend(&other);
        // };
        // if let Some(other) = self.clients.get_mut(other).unwrap().as_mut() {
        //     other.add_friend(&me);
        // };
    }

    pub async fn send(&mut self, name: &str, message: &str) {
        let node = *self.nodes.get(name).unwrap();
        self.network[node].send(message).await;
    }
}
