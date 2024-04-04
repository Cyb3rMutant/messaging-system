// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::BufReader;
use std::{
    net::TcpStream,
    sync::{Arc, Mutex, RwLock},
};

use app::chats::Chats;
use tauri::Manager;

fn main() {
    let stream = TcpStream::connect("127.0.0.1:8080").unwrap();

    let writer = stream.try_clone().unwrap();

    let reader = BufReader::new(stream);

    use app::from_server::*;
    use app::internal_commands::*;
    use app::to_server::*;
    let chats = Chats::new();
    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();

            tauri::async_runtime::spawn(read_messages(app_handle, reader));

            Ok(())
        })
        .manage(Sender(Arc::new(Mutex::new(writer))))
        .manage(GlobalChats(Arc::new(RwLock::new(chats))))
        .invoke_handler(tauri::generate_handler![
            send,
            getusers,
            login,
            register,
            switch_chat,
            read_chat,
            delete,
            update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
