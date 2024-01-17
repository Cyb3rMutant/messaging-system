use std::{
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
};

use tauri::State;

use crate::from_server::GlobalChats;

pub struct Sender(pub Arc<Mutex<TcpStream>>);

#[tauri::command]
pub fn send(
    user: String,
    message: String,
    sender: State<'_, Sender>,
    chats: State<'_, GlobalChats>,
) {
    let mut writer = sender.0.lock().unwrap();

    writer
        .write_all(format!("SND;{};{}\n", user, message).as_bytes())
        .expect("Failed to send message to the server");

    chats.0.write().unwrap().sent_message(&user, message);
}

#[tauri::command]
pub fn getusers(sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("GET;\n").as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
pub fn login(username: String, password: String, sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("{};{}\n", username, password).as_bytes())
        .expect("Failed to send message to the server");
}
