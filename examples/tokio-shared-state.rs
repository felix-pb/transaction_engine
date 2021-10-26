use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use transaction_engine::{ClientId, TransactionEngine};

const NUMBER_OF_TRANSACTIONS: u32 = 65_536_000;

#[tokio::main]
async fn main() {
    let engine = Arc::new(Mutex::new(TransactionEngine::init()));

    let now = Instant::now();

    let task = tokio::spawn(async move {
        for tx in 0..NUMBER_OF_TRANSACTIONS {
            let client = (tx % 65_536) as ClientId;
            if tx < NUMBER_OF_TRANSACTIONS / 2 {
                engine.lock().deposit(client, tx, 1).unwrap();
            } else {
                engine.lock().withdrawal(client, tx, 1).unwrap();
            }
        }
    });

    task.await.unwrap();

    dbg!(now.elapsed());
}
