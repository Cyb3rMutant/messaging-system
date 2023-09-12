use ms::{manager, process};
use tokio::{net::TcpListener, sync::mpsc};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Server listening on 127.0.0.1:8080");

    let (tx, rx) = mpsc::channel(32);

    tokio::spawn(async move {
        manager(rx).await;
    });

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        println!("{stream:?} {addr:?}");
        let tx = tx.clone();

        tokio::spawn(async move {
            process(stream, tx).await;
        });
    }
}
