use petgraph::{
    graph::{NodeIndex, UnGraph},
    visit::{EdgeRef, IntoNodeReferences},
    Graph, Undirected,
};
use tokio::io::{AsyncWriteExt, WriteHalf};

use crate::{client::Client, message::Message};
use std::collections::HashMap;

pub struct Container<T: AsyncWriteExt> {
    nodes: HashMap<i32, NodeIndex>,
    network: Graph<Client<T>, i32, Undirected>,
}

impl<T: AsyncWriteExt> Container<T> {
    pub fn new(users: Vec<(i32, String, i32, String, i32)>, lonely: Vec<(i32, String)>) -> Self {
        let mut nodes = HashMap::new();
        let mut network = UnGraph::new_undirected();
        println!("{users:?} {lonely:?}");
        for (id, username) in lonely {
            if !nodes.contains_key(&id) {
                let n = network.add_node(Client::new(id, username));
                nodes.insert(id, n);
            }
        }
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

    pub fn new_user(&mut self, id: i32, name: String) {
        let n = self.network.add_node(Client::new(id, name));

        self.nodes.insert(id, n);
    }

    // fn push(&mut self, client: Client) {
    //     let c = self.clients.get_mut(&client.name).unwrap();
    //     *c = Some(client);
    // }

    pub async fn login(
        &mut self,
        id: i32,
        writer: WriteHalf<T>,
        messages: Vec<Message>,
        ids: Vec<i32>,
    ) -> Result<(), WriteHalf<T>> {
        let messages = serde_json::to_string(&messages).unwrap();
        let ids = serde_json::to_string(&ids).unwrap();
        let message = format!("LGN;{};{};{}\n", id, ids, messages);
        let node = *self.nodes.get(&id).unwrap();
        match self.network[node].login(writer) {
            Ok(_) => {
                self.send(id, &message).await;
                Ok(())
            }
            Err(w) => Err(w),
        }
    }

    pub fn remove(&mut self, id: i32) {
        let node = *match self.nodes.get(&id) {
            Some(n) => n,
            None => return,
        };
        self.network[node].loguot();
    }

    fn get_friends(&self, id: i32) -> String {
        let mut list = String::new();

        let node = *self.nodes.get(&id).unwrap();
        for e in self.network.edges(node) {
            let id = e.weight();
            let u = self.network.edge_endpoints(e.id()).unwrap();
            let u = if u.0 == node { u.1 } else { u.0 };
            println!("{:?}", u);
            list += &format!(";{};{}", id, self.network[u].name);
        }

        println!("{list:?}");
        if list.is_empty() {
            list += ";";
        }
        list
    }

    pub async fn send_friends(&mut self, id: i32) {
        // self.send(name, &format!("USR{}\n", self.get_all())).await;
        self.send(id, &format!("FRD{}\n", self.get_friends(id)))
            .await;
    }
    pub async fn get_name(&self, id: i32) -> String {
        let node = *self.nodes.get(&id).unwrap();
        self.network[node].name.clone()
    }

    pub fn add_friends(&mut self, id: i32, other: i32, chat_id: i32) {
        let n1 = *self.nodes.get(&id).unwrap();
        let n2 = *self.nodes.get(&other).unwrap();
        self.network.add_edge(n1, n2, chat_id);
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
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.network.clear();
    }

    pub async fn send(&mut self, id: i32, message: &str) {
        let node = *self.nodes.get(&id).unwrap();
        self.network[node].send(message).await;
    }
}
