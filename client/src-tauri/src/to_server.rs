use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use tauri::State;

use crate::{
    from_server::GlobalChats,
    hashing::{modular_pow, xor_encrypt},
};

pub struct Sender(pub Arc<Mutex<TcpStream>>);

#[tauri::command]
pub fn send(user: i32, message: String, sender: State<'_, Sender>, chats: State<'_, GlobalChats>) {
    let mut writer = sender.0.lock().unwrap();

    let c = chats.0.write().unwrap();
    let s_message = xor_encrypt(
        &message,
        modular_pow(c.get_b(user) as u64, c.get_a() as u64, user as u64) as i32,
    );
    // drop(chats);

    writer
        .write_all(format!("SND;{};{}\n", user, s_message).as_bytes())
        .expect("Failed to send message to the server");

    chats.0.write().unwrap().pend_message(user, message);
}

#[tauri::command]
pub fn read_chat(user: i32, sender: State<'_, Sender>, chats: State<'_, GlobalChats>) {
    println!("in reading chat, {user}");
    chats.0.write().unwrap().other_message_read(user);
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("STS;{user}\n").as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
pub fn get_all(sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("GET;ALL\n").as_bytes())
        .expect("Failed to send message to the server");
}
#[tauri::command]
pub fn get_friends(sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("GET;FRD\n").as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
pub fn register(username: String, password: String, sender: State<'_, Sender>) {
    println!("in register");
    println!("{username}, {password}");
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("REG;{};{}\n", username, password).as_bytes())
        .expect("Failed to send message to the server");
    println!("done");
}

#[tauri::command]
pub fn login(
    username: String,
    password: String,
    sender: State<'_, Sender>,
    chats: State<'_, GlobalChats>,
) {
    chats.0.write().unwrap().set_a(password.len() as i32);
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("LGN;{};{}\n", username, password).as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
pub fn delete(
    user: i32,
    message_id: i32,
    sender: State<'_, Sender>,
    chats: State<'_, GlobalChats>,
) {
    println!("deleting");
    chats.0.write().unwrap().delete(user, message_id);
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("DEL;{user};{message_id}\n").as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
pub fn update(
    user: i32,
    message_id: i32,
    content: String,
    sender: State<'_, Sender>,
    chats: State<'_, GlobalChats>,
) {
    println!("updating");
    chats.0.write().unwrap().update(user, message_id, &content);
    let mut writer = sender.0.lock().unwrap();
    let c = chats.0.write().unwrap();
    let content = xor_encrypt(
        &content,
        modular_pow(c.get_b(user) as u64, c.get_a() as u64, user as u64) as i32,
    );
    // drop(c);
    writer
        .write_all(format!("UPD;{user};{message_id};{content}\n").as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
pub fn connect(id: i32, sender: State<'_, Sender>) {
    println!("connecting");
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("CNT;{id}\n").as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
pub fn send_a(user: i32, a: i32, sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("A;{user};{a}\n").as_bytes())
        .expect("Failed to send message to the server");
}
