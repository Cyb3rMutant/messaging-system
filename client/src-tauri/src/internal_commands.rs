use std::collections::VecDeque;

use tauri::State;

use crate::{chats::Message, from_server::GlobalChats};

#[tauri::command]
pub fn switch_chat(user: String, chats: State<'_, GlobalChats>) -> VecDeque<Message> {
    let chats = chats.0.read().unwrap();
    chats.get_chat(&user).clone()
}
