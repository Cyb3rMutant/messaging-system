// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};

async fn read_messages(app: AppHandle, stream: TcpStream) {
    let mut reader = BufReader::new(&stream);

    loop {
        let mut buf = String::new();
        if reader.read_line(&mut buf).unwrap() == 0 {
            break;
        }
        let _ = app.emit_all("received", buf);
    }
}

struct Sender(Arc<Mutex<TcpStream>>);

#[tauri::command]
fn send(message: String, sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();

    let mut message = message;
    message.push('\n');

    writer
        .write_all(message.as_bytes())
        .expect("Failed to send message to the server");
}

fn main() {
    let stream = TcpStream::connect("127.0.0.1:8080").unwrap();

    let mut writer = stream
        .try_clone()
        .expect("Failed to clone stream for writing");

    let mut reader = BufReader::new(&stream);
    reader
        .read_line(&mut String::new())
        .expect("Failed to read welcome message");

    writer
        .write_all("tester\n".as_bytes())
        .expect("Failed to send the name to the server");

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();

            tauri::async_runtime::spawn(read_messages(app_handle, stream));

            Ok(())
        })
        .manage(Sender(Arc::new(Mutex::new(writer))))
        .invoke_handler(tauri::generate_handler![send])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
