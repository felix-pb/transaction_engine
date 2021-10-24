use std::time::Instant;
use transaction_engine::TransactionEngine;

const NUM_TRANSACTIONS: u32 = 100_000_000;

fn main() {
    let mut engine = TransactionEngine::init();

    let now = Instant::now();

    for tx in 0..NUM_TRANSACTIONS {
        engine.deposit(1, tx, 1).unwrap();
    }

    let elapsed = now.elapsed();

    let client1 = engine.get_account(1).unwrap();
    assert_eq!(NUM_TRANSACTIONS as i64, client1.get_available_balance());
    assert_eq!(0, client1.get_held_balance());
    assert_eq!(NUM_TRANSACTIONS as i64, client1.get_total_balance());
    assert!(!client1.is_locked());

    dbg!(elapsed);
}
