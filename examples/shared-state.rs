use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use transaction_engine::{ClientId, TransactionEngine};

const NUMBER_OF_CLIENTS: u32 = 65_536;
const NUMBER_OF_THREADS: u32 = 10;
const NUMBER_OF_TRANSACTIONS: u32 = 6_553_600;

fn main() {
    let engine = Arc::new(Mutex::new(TransactionEngine::init()));

    let now = Instant::now();

    let mut threads = Vec::with_capacity(10);
    for i in 0..NUMBER_OF_THREADS {
        let engine = engine.clone();
        let thread = std::thread::spawn(move || {
            for tx in 0..NUMBER_OF_TRANSACTIONS {
                let client = (tx % NUMBER_OF_CLIENTS) as ClientId;
                let unique_tx = i * NUMBER_OF_TRANSACTIONS + tx;
                if tx < NUMBER_OF_TRANSACTIONS / 2 {
                    engine.lock().deposit(client, unique_tx, 1).unwrap();
                } else {
                    engine.lock().withdrawal(client, unique_tx, 1).unwrap();
                }
            }
        });
        threads.push(thread);
    }

    threads
        .into_iter()
        .for_each(|thread| thread.join().unwrap());

    dbg!(now.elapsed());
}
