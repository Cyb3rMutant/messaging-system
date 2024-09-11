use server::{client_process::Process, manager::Manager};
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").await.unwrap();
    println!("Server listening on 0.0.0.0:7878");

    let (tx, rx) = mpsc::channel(32);

    let mut manager = Manager::new(rx).await;
    tokio::spawn(async move {
        manager.run().await;
    });

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("{stream:?} {addr:?}");
        let tx = tx.clone();

        tokio::spawn(async move {
            Process::run(stream, tx).await;
        });
    }
}
