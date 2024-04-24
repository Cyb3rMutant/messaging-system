use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::hashing::{modular_pow, xor_encrypt};

#[derive(Debug, Serialize, Clone)]
pub struct Message {
    pub message_id: i32,
    pub from_me: bool,
    pub content: String,
    pub status: i32,
}

#[derive(Debug, Deserialize)]
struct ServerMessage {
    message_id: i32,
    chat_id: i32,
    sender_id: i32,
    content: String,
    status: i32,
}

#[derive(Debug)]
pub struct Chats {
    me: i32,
    chats: HashMap<i32, (VecDeque<Message>, i32, i32)>,
    pending_messages: VecDeque<(i32, Message)>,
    a: i32,
}

impl Chats {
    pub fn new() -> Chats {
        Chats {
            me: i32::default(),
            chats: HashMap::new(),
            pending_messages: VecDeque::new(),
            a: 0,
        }
    }

    pub fn set_id(&mut self, id: i32) {
        self.me = id;
    }
    pub fn set_a(&mut self, a: i32) {
        self.a = a;
    }
    pub fn set_b(&mut self, user: i32, b: i32) {
        self.chats.get_mut(&user).unwrap().2 = b;
    }
    pub fn get_a(&self) -> i32 {
        self.a
    }
    pub fn get_b(&self, user: i32) -> i32 {
        self.chats.get(&user).unwrap().2
    }

    pub fn add_chat(&mut self, chat_id: i32, g: i32, b: i32) {
        if self.chats.contains_key(&chat_id) {
            return;
        }
        self.chats.insert(chat_id, (VecDeque::new(), g, b));
    }

    fn add_message(
        &mut self,
        user: i32,
        message_id: i32,
        content: String,
        from_me: bool,
        status: i32,
    ) -> Message {
        let message = Message {
            message_id,
            from_me,
            content,
            status,
        };
        println!("{:?}", self.chats);
        match self.chats.get_mut(&user) {
            Some((chat, _g, _b)) => {
                chat.push_back(message.clone());
            }
            None => {
                println!("im smart");
            }
        }
        message
    }
    pub fn pend_message(&mut self, user: i32, content: String) {
        println!("p 1 ");
        let message = Message {
            message_id: 0,
            from_me: true,
            content,
            status: 1,
        };
        println!("p 2 ");
        self.pending_messages.push_back((user, message));
        println!("p 3 ");
    }

    pub fn sent_message(&mut self, message_id: i32) -> (i32, Message) {
        let (user, mut m) = self.pending_messages.pop_front().unwrap();
        m.message_id = message_id;
        self.chats.get_mut(&user).unwrap().0.push_back(m.clone());
        (user, m)
    }

    pub fn received_message(&mut self, user: i32, message_id: i32, content: String) -> Message {
        self.add_message(user, message_id, content, false, 1)
    }

    pub fn get_chat(&self, user: i32) -> &VecDeque<Message> {
        println!("{:?}", self.chats.get(&user).unwrap());
        &self.chats.get(&user).unwrap().0
    }

    pub fn is_me(&self, user: i32) -> bool {
        self.me == user
    }

    pub fn load(&mut self, messages: &str, chats: &str) {
        let messages: Vec<ServerMessage> = serde_json::from_str(messages).unwrap();
        let empty_chats: Vec<(i32, i32, i32)> = serde_json::from_str(chats).unwrap();
        for (p, g, b) in empty_chats {
            self.add_chat(p, g, b);
        }
        for m in messages.into_iter() {
            self.add_message(
                m.chat_id,
                m.message_id,
                xor_encrypt(
                    &m.content,
                    modular_pow(
                        self.get_b(m.chat_id) as u64,
                        self.get_a() as u64,
                        m.chat_id as u64,
                    ) as i32,
                ),
                self.is_me(m.sender_id),
                m.status,
            );
        }
    }
    pub fn my_message_read(&mut self, user: i32) {
        self.set_read(user, true)
    }
    pub fn other_message_read(&mut self, user: i32) {
        self.set_read(user, false)
    }
    fn set_read(&mut self, user: i32, from_me: bool) {
        println!("{:?} {:?}", self.chats, user);
        for m in self.chats.get_mut(&user).unwrap().0.iter_mut().rev() {
            if m.from_me != from_me {
                continue;
            }
            if m.status == 2 {
                break;
            }
            m.status = 2;
        }
    }
    pub fn delete(&mut self, user: i32, message_id: i32) {
        for m in self.chats.get_mut(&user).unwrap().0.iter_mut().rev() {
            if m.message_id != message_id {
                continue;
            }
            m.content = "".to_owned();
            m.status = 3;
            break;
        }
    }
    pub fn update(&mut self, user: i32, message_id: i32, content: &str) {
        for m in self.chats.get_mut(&user).unwrap().0.iter_mut().rev() {
            if m.message_id != message_id {
                continue;
            }
            m.content = content.to_owned();
            m.status = 4;
            break;
        }
    }
}
