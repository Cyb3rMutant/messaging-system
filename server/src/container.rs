use petgraph::{
    graph::{NodeIndex, UnGraph},
    visit::EdgeRef,
    Graph, Undirected,
};
use tokio::{io::WriteHalf, net::TcpStream};

use crate::{client::Client, message::Message};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Container {
    nodes: HashMap<i32, NodeIndex>,
    network: Graph<Client, i32, Undirected>,
}

impl Container {
    pub fn new(users: Vec<(i32, String, i32, String, i32)>) -> Self {
        let mut nodes = HashMap::new();
        let mut network = UnGraph::new_undirected();

        for (id_1, username_1, id_2, username_2, chat_id) in users {
            if !nodes.contains_key(&id_1) {
                let n = network.add_node(Client::new(id_1, username_1));
                nodes.insert(id_1, n);
            }

            if !nodes.contains_key(&id_2) {
                let n = network.add_node(Client::new(id_2, username_2));
                nodes.insert(id_2, n);
            }

            let node_1 = nodes.get(&id_1).unwrap();
            let node_2 = nodes.get(&id_2).unwrap();
            network.add_edge(*node_1, *node_2, chat_id);
        }
        // clients: users.into_iter().map(|u| (u, None)).collect(),
        Container { nodes, network }
    }

    // fn push(&mut self, client: Client) {
    //     let c = self.clients.get_mut(&client.name).unwrap();
    //     *c = Some(client);
    // }

    pub async fn login(&mut self, id: i32, writer: WriteHalf<TcpStream>, messages: Vec<Message>) {
        let messages = serde_json::to_string(&messages).unwrap();
        let message = format!("LGN;{};{}\n", id, messages);
        let node = *self.nodes.get(&id).unwrap();
        self.network[node].login(writer);
        self.send(id, &message).await;
    }

    pub fn remove(&mut self, id: i32) {
        let node = *self.nodes.get(&id).unwrap();
        self.network[node].loguot();
    }

    pub fn get_all(&self) -> String {
        let mut list = String::new();

        for c in self.nodes.keys() {
            list.push(';');
            list += &c.to_string();
        }
        println!("{list:?}");

        list
    }
    fn get_friends(&self, id: i32) -> String {
        let mut list = String::new();

        let node = *self.nodes.get(&id).unwrap();
        // for u in self.network.neighbors(node) {
        //     list.push(';');
        //     list += &self.network[u].name;
        // }
        for e in self.network.edges(node) {
            let id = e.weight();
            let u = self.network.edge_endpoints(e.id()).unwrap();
            let u = if u.0 == node { u.1 } else { u.0 };
            println!("{:?}", u);
            list += &format!(";{};{}", id, self.network[u].name);
        }

        println!("{list:?}");
        list
    }

    pub async fn send_users(&mut self, id: i32) {
        // self.send(name, &format!("USR{}\n", self.get_all())).await;
        self.send(id, &format!("USR{}\n", self.get_friends(id)))
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
    pub fn get_other(&self, chat_id: i32, first_id: i32) -> i32 {
        let node = *self.nodes.get(&first_id).unwrap();
        for e in self.network.edges(node) {
            if &chat_id != e.weight() {
                continue;
            }
            let u = self.network.edge_endpoints(e.id()).unwrap();
            let u = if u.0 == node { u.1 } else { u.0 };
            return self.network[u].id;
        }
        i32::default()
    }

    pub async fn send(&mut self, id: i32, message: &str) {
        let node = *self.nodes.get(&id).unwrap();
        self.network[node].send(message).await;
    }
}
