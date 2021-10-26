use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use transaction_engine::{ClientId, TransactionEngine};

const NUMBER_OF_CLIENTS: u32 = 65_536;
const NUMBER_OF_THREADS: u32 = 10;
const NUMBER_OF_TRANSACTIONS: u32 = 6_553_600;

fn main() {
    // Initialize the transaction engine.
    let engine = Arc::new(Mutex::new(TransactionEngine::init()));

    // Start the timer.
    let now = Instant::now();

    // Process 6.5536 million transactions in each worker thread.
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

    // Wait for all worker threads to finish.
    threads
        .into_iter()
        .for_each(|thread| thread.join().unwrap());

    // Stop the timer.
    let elapsed = now.elapsed();

    // Validate that each client account is empty and unlocked.
    let engine = engine.lock();
    for client in ClientId::MIN..=ClientId::MAX {
        let account = engine.get_account(client).unwrap();
        assert_eq!(0, account.get_available_balance());
        assert_eq!(0, account.get_held_balance());
        assert_eq!(0, account.get_total_balance());
        assert!(!account.is_locked());
    }

    // Print the result of the benchmark.
    dbg!(elapsed);
}
