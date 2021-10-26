use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use transaction_engine::TransactionEngine;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut engine = TransactionEngine::init();
        // blocking here is okay because this thread is not running in the tokio
        // worker thread pool. in real scenario, this thread should be pinned to a core.
        while let Ok(_transaction) = rx.recv() {
            // depending on the transaction, call the approprate engine method
            engine.deposit(0, 0, 0).unwrap();
        }
    });
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let _n = stream.read(&mut buffer).await.unwrap();
                // try to parse a transaction from the input data.
                // then send the transaction structure to the engine thread.
                // note that `send` is not a blocking method even if this
                // is the std's mpsc channel and not the tokio version.
                tx.send("transaction structure").unwrap();
            }
        });
    }
}
