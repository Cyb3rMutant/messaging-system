// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use tauri::{AppHandle, Manager, State};

async fn read_messages(app: AppHandle, mut reader: BufReader<TcpStream>) {
    loop {
        let mut buf = String::new();
        if reader.read_line(&mut buf).unwrap() == 0 {
            break;
        }
        println!("{:?}", buf);
        let (command, content) = buf.trim().split_once(';').unwrap();
        let _ = match command {
            "MSG" => app.emit_all("MSG", content.replace(";", ": ")),
            "USR" => app.emit_all("USR", content.split(";").collect::<Vec<&str>>()),
            "LGN" => app.emit_all("LGN", true),
            _ => app.emit_all("OTH", content),
        };
    }
}

struct Sender(Arc<Mutex<TcpStream>>);

#[tauri::command]
fn send(user: String, message: String, sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();

    writer
        .write_all(format!("SND;{};{}\n", user, message).as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
fn getusers(sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("GET;\n").as_bytes())
        .expect("Failed to send message to the server");
}

#[tauri::command]
fn login(username: String, sender: State<'_, Sender>) {
    let mut writer = sender.0.lock().unwrap();
    writer
        .write_all(format!("{}\n", username).as_bytes())
        .expect("Failed to send message to the server");
}

fn main() {
    let stream = TcpStream::connect("127.0.0.1:8080").unwrap();

    let writer = stream.try_clone().unwrap();

    let reader = BufReader::new(stream);

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();

            tauri::async_runtime::spawn(read_messages(app_handle, reader));

            Ok(())
        })
        .manage(Sender(Arc::new(Mutex::new(writer))))
        .invoke_handler(tauri::generate_handler![send, getusers, login])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
