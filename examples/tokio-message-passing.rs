use std::time::Instant;
use transaction_engine::{ClientId, TransactionEngine};

const NUMBER_OF_CLIENTS: u32 = 65_536;
const NUMBER_OF_TRANSACTIONS: u32 = 65_536_000;

#[tokio::main]
async fn main() {
    let mut engine = TransactionEngine::init();
    let (sender, mut receiver) = tokio::sync::mpsc::channel(NUMBER_OF_TRANSACTIONS as usize);

    let now = Instant::now();

    let task1 = tokio::spawn(async move {
        while let Some((deposit, client, tx, amount)) = receiver.recv().await {
            if deposit {
                engine.deposit(client, tx, amount).unwrap();
            } else {
                engine.withdrawal(client, tx, amount).unwrap();
            }
        }
    });

    let task2 = tokio::spawn(async move {
        for tx in 0..NUMBER_OF_TRANSACTIONS {
            let client = (tx % NUMBER_OF_CLIENTS) as ClientId;
            if tx < NUMBER_OF_TRANSACTIONS / 2 {
                sender.send((true, client, tx, 1)).await.unwrap();
            } else {
                sender.send((false, client, tx, 1)).await.unwrap();
            }
        }
    });

    task1.await.unwrap();
    task2.await.unwrap();

    dbg!(now.elapsed());
}
