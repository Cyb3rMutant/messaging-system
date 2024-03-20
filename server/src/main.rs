use std::{sync::Arc, time::Duration};

use server::{client_process::Process, manager::Manager};
use tokio::{net::TcpListener, sync::mpsc, time::sleep};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server listening on 127.0.0.1:8080");

    let (tx, rx) = mpsc::channel(32);

    let manager = Arc::new(Manager::new(rx).await);
    let c_manager = Arc::clone(&manager);
    tokio::spawn(async move {
        manager.run().await;
    });

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("{stream:?} {addr:?}");
        let tx = tx.clone();

        sleep(Duration::from_secs(2)).await;
        let manager = Arc::clone(&c_manager);
        tokio::spawn(async move {
            Process::run(stream, tx, manager).await;
        });
    }
}
