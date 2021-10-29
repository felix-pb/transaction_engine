use std::time::Instant;
use transaction_engine::{ClientId, TransactionEngine};

const NUMBER_OF_CLIENTS: u32 = 65_536;
const NUMBER_OF_TRANSACTIONS: u32 = 65_536_000;

fn main() {
    // Initialize the transaction engine.
    let mut engine = TransactionEngine::init();

    // Start the timer.
    let now = Instant::now();

    // Process 65.536 million transactions by going through each client account in a round-robin
    // manner 1000 times. Make deposits for the first half and withdrawals for the second half.
    for tx in 0..NUMBER_OF_TRANSACTIONS {
        let client = (tx % NUMBER_OF_CLIENTS) as ClientId;
        if tx < NUMBER_OF_TRANSACTIONS / 2 {
            engine.deposit(client, tx, 1).unwrap();
        } else {
            engine.withdrawal(client, tx, 1).unwrap();
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

// for client in 0..NUMBER_OF_CLIENTS {
//     let id = client as ClientId;
//     for tx in 0..1000 {
//         if tx < 500 {
//             engine.deposit(id, tx * NUMBER_OF_CLIENTS + client, 1).unwrap(); // <-- BAD CACHING
//             engine.deposit(id, tx + client * 1000, 1).unwrap();              // <-- GOOD CACHING
//         } else {
//             engine.withdrawal(id, tx * NUMBER_OF_CLIENTS + client, 1).unwrap();  // <-- BAD CACHING
//             engine.withdrawal(id, tx + client * 1000, 1).unwrap();               // <-- GOOD CACHING
//         }
//     }
// }
