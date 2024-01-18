use std::{sync::Arc, time::Duration};

use server::{manager::Manager, process};
use tokio::{net::TcpListener, sync::mpsc, time::sleep};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server listening on 127.0.0.1:8080");

    let (tx, rx) = mpsc::channel(32);

    let manager = Arc::new(Manager::new(rx).await);
    let _c_manager = Arc::clone(&manager);
    tokio::spawn(async move {
        manager.run().await;
    });

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("{stream:?} {addr:?}");
        let tx = tx.clone();

        sleep(Duration::from_secs(2)).await;
        tokio::spawn(async move {
            process(stream, tx).await;
        });
    }
}
