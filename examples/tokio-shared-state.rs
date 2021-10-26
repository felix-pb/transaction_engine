use std::sync::{Arc, Mutex};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use transaction_engine::TransactionEngine;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let engine = Arc::new(Mutex::new(TransactionEngine::init()));
    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        let engine = engine.clone();
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            loop {
                let _n = stream.read(&mut buffer).await.unwrap();
                // try to parse a transaction from the input data.
                // then call appropriate engine method by locking the mutex
                // but hold the guard for as short a time as possible.
                engine.lock().unwrap().deposit(0, 0, 0).unwrap();
            }
        });
    }
}
