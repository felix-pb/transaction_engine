use std::time::Instant;
use transaction_engine::{ClientId, TransactionEngine};

const NUMBER_OF_CLIENTS: u32 = 65_536;
const NUMBER_OF_THREADS: u32 = 10;
const NUMBER_OF_TRANSACTIONS: u32 = 6_553_600;

fn main() {
    // Initialize the transaction engine.
    let mut engine = TransactionEngine::init();
    let (sender, receiver) = std::sync::mpsc::channel();

    // Start the timer.
    let now = Instant::now();

    // Send 6.5536 million transactions from each worker thread.
    for i in 0..NUMBER_OF_THREADS {
        let sender = sender.clone();
        std::thread::spawn(move || {
            for tx in 0..NUMBER_OF_TRANSACTIONS {
                let client = (tx % NUMBER_OF_CLIENTS) as ClientId;
                let unique_tx = i * NUMBER_OF_TRANSACTIONS + tx;
                if tx < NUMBER_OF_TRANSACTIONS / 2 {
                    sender.send((true, client, unique_tx, 1)).unwrap();
                } else {
                    sender.send((false, client, unique_tx, 1)).unwrap();
                }
            }
        });
    }

    drop(sender);

    // Process all received transactions in a single thread.
    while let Ok((deposit, client, tx, amount)) = receiver.recv() {
        if deposit {
            engine.deposit(client, tx, amount).unwrap();
        } else {
            engine.withdrawal(client, tx, amount).unwrap();
        }
    }

    // Stop the timer.
    let elapsed = now.elapsed();

    // Validate that each client account is empty and unlocked.
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
