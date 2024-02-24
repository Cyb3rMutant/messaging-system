use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
    sync::{Arc, RwLock},
};

use tauri::{AppHandle, Manager};

use crate::chats::Chats;

pub struct GlobalChats(pub Arc<RwLock<Chats>>);

pub async fn read_messages(app: AppHandle, mut reader: BufReader<TcpStream>) {
    loop {
        let mut buf = String::new();
        if reader.read_line(&mut buf).unwrap() == 0 {
            break;
        }
        let (command, content) = buf.trim().split_once(';').unwrap();
        let _ = match command {
            "MSG" => receive(content, &app),
            "USR" => users(content, &app),
            "LGN" => logged_in(content, &app),
            "REG" => app.emit_all("REG", content).unwrap(),
            "ERR" => app.emit_all("ERR", content).unwrap(),
            _ => app.emit_all("OTH", content).unwrap(),
        };
    }
}

fn receive(content: &str, app: &AppHandle) {
    let (id, message) = content.split_once(';').unwrap();
    let id = id.parse::<i32>().unwrap();
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    if chats.is_me(id) {
        return;
    }
    let message = chats.received_message(id, message.to_owned());

    app.emit_all("MSG", (id, message)).unwrap();
}

fn users<'a>(content: &'a str, app: &AppHandle) {
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    let users = content.split(";").collect::<Vec<&str>>();
    let mut iter = users.iter();
    while let Some(chat_id) = iter.next() {
        if let Some(name) = iter.next() {
            println!("{:?} {:?}", chat_id, name);
            chats.add_chat(chat_id.parse().unwrap());
        }
    }

    // for user in users.iter() {
    //     chats.add_chat(user.to_string());
    // }

    app.emit_all("USR", users).unwrap();
}

fn logged_in(content: &str, app: &AppHandle) {
    let (id, messages) = content.split_once(';').unwrap();
    let id = id.parse::<i32>().unwrap();
    println!("{}\t\t{}", id, messages);
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    chats.set_id(id);

    app.emit_all("LGN", id).unwrap();
}
