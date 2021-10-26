use std::time::Instant;
use transaction_engine::{ClientId, TransactionEngine};

const NUMBER_OF_CLIENTS: u32 = 65_536;
const NUMBER_OF_THREADS: u32 = 4;
const NUMBER_OF_TRANSACTIONS: u32 = 655_360;

#[tokio::main]
async fn main() {
    let mut engine = TransactionEngine::init();
    let (sender, receiver) = std::sync::mpsc::channel();

    let now = Instant::now();

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

    while let Ok((deposit, client, tx, amount)) = receiver.recv() {
        if deposit {
            engine.deposit(client, tx, amount).unwrap();
        } else {
            engine.withdrawal(client, tx, amount).unwrap();
        }
    }

    dbg!(now.elapsed());
}
