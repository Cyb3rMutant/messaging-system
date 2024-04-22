use std::{
    io::{BufRead, BufReader},
    net::TcpStream,
    sync::{Arc, RwLock},
};

use tauri::{AppHandle, Manager};

use crate::{
    chats::{Chats, Message},
    hashing::{modular_pow, xor_encrypt},
};

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
            "CNT" => connect(content, &app),
            "FRD" => friends(content, &app),
            "ALL" => all(content, &app),
            "LGN" => logged_in(content, &app),
            "STS" => set_seen(content, &app),
            "DEL" => delete(content, &app),
            "UPD" => update(content, &app),
            "MID" => message_sent(content, &app),
            "B" => b(content, &app),
            "REG" => app.emit_all("REG", content).unwrap(),
            "ERR" => app.emit_all("ERR", content).unwrap(),

            _ => app.emit_all("OTH", content).unwrap(),
        };
    }
}
fn receive(content: &str, app: &AppHandle) {
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    let args: Vec<&str> = content.split(';').collect();
    let id: i32 = args.get(0).unwrap().parse().unwrap();
    let message_id: i32 = args.get(1).unwrap().parse().unwrap();
    let content = args.get(2).unwrap();
    println!("{content}");
    let content = xor_encrypt(
        &content,
        modular_pow(chats.get_b(id) as u64, chats.get_a() as u64, id as u64) as i32,
    );
    println!("{content}");

    let message = chats.received_message(id, message_id, content);

    app.emit_all("MSG", (id, message)).unwrap();
}
fn connect(content: &str, app: &AppHandle) {
    println!("{content}");
    let args: Vec<&str> = content.split(';').collect();
    let chat_id: i32 = args.get(0).unwrap().parse().unwrap();
    let user_id: i32 = args.get(1).unwrap().parse().unwrap();
    let g: i32 = args.get(2).unwrap().parse().unwrap();
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    chats.add_chat(chat_id, g, 0);
    let a = modular_pow(g as u64, chats.get_a() as u64, chat_id as u64) as i32;

    app.emit_all("CNT", (chat_id, user_id, a)).unwrap();
}

fn friends<'a>(content: &'a str, app: &AppHandle) {
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    let users = content.split(";").collect::<Vec<&str>>();
    // let mut iter = users.iter();
    // while let Some(chat_id) = iter.next() {
    //     if let Some(name) = iter.next() {
    //         println!("{:?} {:?}", chat_id, name);
    //         chats.add_chat(chat_id.parse().unwrap());
    //     }
    // }

    app.emit_all("FRD", users).unwrap();
}
fn all<'a>(content: &'a str, app: &AppHandle) {
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    let users = content.split(";").collect::<Vec<&str>>();
    // let mut iter = users.iter();
    // while let Some(chat_id) = iter.next() {
    //     if let Some(name) = iter.next() {
    //         println!("{:?} {:?}", chat_id, name);
    //         chats.add_chat(chat_id.parse().unwrap());
    //     }
    // }

    app.emit_all("ALL", users).unwrap();
}

fn logged_in(content: &str, app: &AppHandle) {
    let (id, chats) = content.split_once(';').unwrap();
    let (chats_p_g, messages) = chats.split_once(';').unwrap();
    let id = id.parse::<i32>().unwrap();
    println!("{}\t\t{}\t\t{chats_p_g}", id, messages);
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    chats.set_id(id);
    chats.load(messages, chats_p_g);

    app.emit_all("LGN", id).unwrap();
}

fn set_seen(content: &str, app: &AppHandle) {
    let chat_id: i32 = content.parse().unwrap();

    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    chats.my_message_read(chat_id);

    app.emit_all("STS", chat_id).unwrap();
}

fn delete(content: &str, app: &AppHandle) {
    let (id, message_id) = content.split_once(';').unwrap();

    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    chats.delete(id.parse().unwrap(), message_id.parse().unwrap());

    app.emit_all("DEL", (id, message_id)).unwrap();
}

fn update(content: &str, app: &AppHandle) {
    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    let args: Vec<&str> = content.split(';').collect();
    let id: i32 = args.get(0).unwrap().parse().unwrap();
    let message_id: i32 = args.get(1).unwrap().parse().unwrap();
    let content = args.get(2).unwrap();
    let content = xor_encrypt(
        &content,
        modular_pow(chats.get_b(id) as u64, chats.get_a() as u64, id as u64) as i32,
    );
    chats.update(id, message_id, &content);

    app.emit_all("UPD", (id, message_id, content)).unwrap();
}

fn message_sent(content: &str, app: &AppHandle) {
    println!("message sent 1 ");
    let message_id: i32 = content.parse().unwrap();
    println!("message sent 2 ");
    let state = app.state::<GlobalChats>();
    println!("message sent 3 ");
    let mut chats = state.0.write().unwrap();
    println!("message sent 4 ");
    let (user, message) = chats.sent_message(message_id);
    println!("message sent 5 ");

    app.emit_all("MID", (user, message)).unwrap();
    println!("message sent 6 ");
}

fn b(content: &str, app: &AppHandle) {
    let (chat_id, b) = content.split_once(';').unwrap();
    let chat_id: i32 = chat_id.parse().unwrap();
    let b: i32 = b.parse().unwrap();

    let state = app.state::<GlobalChats>();
    let mut chats = state.0.write().unwrap();
    chats.set_b(chat_id, b);
}
