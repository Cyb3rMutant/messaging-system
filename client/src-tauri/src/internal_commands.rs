use std::collections::VecDeque;

use tauri::State;

use crate::{chats::Message, from_server::GlobalChats};

#[tauri::command]
pub fn switch_chat(user: i32, chats: State<'_, GlobalChats>) -> VecDeque<Message> {
    let chats = chats.0.read().unwrap();
    chats.get_chat(user).clone()
}

#[tauri::command]
pub fn search(user: i32, message: String, chats: State<'_, GlobalChats>) -> Vec<i32> {
    let chats = chats.0.read().unwrap();
    let messages = chats.get_chat(user);
    messages
        .iter()
        .filter(|m| m.content.contains(&message))
        .map(|s| s.message_id)
        .collect()
}
