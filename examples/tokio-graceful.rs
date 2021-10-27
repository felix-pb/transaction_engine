use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver;
use transaction_engine::{Amount, ClientId, TransactionEngine, TransactionId};

const TCP_SERVER_ADDRESS: &str = "127.0.0.1:8080";
const TRANSACTION_BUFFER_SIZE: usize = 10_000;

#[tokio::main]
async fn main() {
    let mut engine = TransactionEngine::init();
    let (sender, mut receiver) = tokio::sync::mpsc::channel(TRANSACTION_BUFFER_SIZE);

    let (watch_tx, watch_rx) = tokio::sync::watch::channel(false);
    let (mpsc_tx, mut mpsx_rx) = tokio::sync::mpsc::channel(1000);
    let mut shutdown = ShutdownTask::new(watch_rx, mpsc_tx);

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        watch_tx.send(true).unwrap();
    });

    // For max performance, pin this thread to an isolated core.
    let engine_shutdown = shutdown.clone();
    std::thread::spawn(move || {
        // For max performance, use `try_recv` in a spin loop instead of `blocking_recv`.
        while let Some(transaction) = receiver.blocking_recv() {
            use Transaction::*;
            let result = match transaction {
                Deposit { client, tx, amount } => engine.deposit(client, tx, amount),
                Withdrawal { client, tx, amount } => engine.withdrawal(client, tx, amount),
                Dispute { client, tx } => engine.dispute(client, tx),
                Resolve { client, tx } => engine.resolve(client, tx),
                Chargeback { client, tx } => engine.chargeback(client, tx),
            };
            if let Err(e) = result {
                eprintln!("transaction error: {}", e);
            }
        }
        println!("beginned shutting down transaction engine");
        std::thread::sleep(Duration::from_secs(1));
        println!("finished shutting down transaction engine");
        drop(engine_shutdown);
    });

    let listener = TcpListener::bind(TCP_SERVER_ADDRESS).await.unwrap();
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, peer) = result.unwrap();
                let sender = sender.clone();
                let mut stream_shutdown = shutdown.clone();
                tokio::spawn(async move {
                    let mut stream = TransactionStream::from(stream);
                    loop {
                        tokio::select! {
                            maybe_transaction = stream.next() => {
                                if let Some(transaction) = maybe_transaction {
                                    sender.send(transaction).await.unwrap();
                                } else {
                                    println!("connection closed by peer: {}", peer);
                                    break;
                                }
                            },
                            _ = stream_shutdown.receiver.changed() => {
                                println!("beginned shutting down connection with {}", peer);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                                println!("finished shutting down connection with {}", peer);
                                break;
                            },
                        };
                    }
                });
            },
            _ = shutdown.receiver.changed() => {
                println!("beginned shutting down tcp listener");
                tokio::time::sleep(Duration::from_secs(1)).await;
                println!("finished shutting down tcp listener");
                break;
            },
        };
    }

    drop(sender);
    drop(shutdown);

    println!("waiting for connections and transaction engine to shutdown gracefully");
    let _ = mpsx_rx.recv().await;
    println!("exiting");
}

#[derive(Clone)]
struct ShutdownTask {
    receiver: Receiver<bool>,
    sender: Sender<()>,
}

impl ShutdownTask {
    fn new(receiver: Receiver<bool>, sender: Sender<()>) -> Self {
        Self { receiver, sender }
    }
}

struct TransactionStream {
    buffer: [u8; 4096],
    stream: TcpStream,
}

impl TransactionStream {
    fn from(stream: TcpStream) -> Self {
        Self {
            buffer: [0; 4096],
            stream,
        }
    }

    async fn next(&mut self) -> Option<Transaction> {
        // Implement the logic to extract a transaction from the socket.
        // Errors can be handled internally, or could be returned to caller.
        let n = self.stream.read(&mut self.buffer).await.unwrap();
        // Do stuff...
        if n == 0 {
            None
        } else {
            Some(Transaction::Deposit {
                client: 1,
                tx: 1,
                amount: 1,
            })
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Transaction {
    Deposit {
        client: ClientId,
        tx: TransactionId,
        amount: Amount,
    },
    Withdrawal {
        client: ClientId,
        tx: TransactionId,
        amount: Amount,
    },
    Dispute {
        client: ClientId,
        tx: TransactionId,
    },
    Resolve {
        client: ClientId,
        tx: TransactionId,
    },
    Chargeback {
        client: ClientId,
        tx: TransactionId,
    },
}
