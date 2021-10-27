use parking_lot::Mutex;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use transaction_engine::{Amount, ClientId, TransactionEngine, TransactionId};

const TCP_SERVER_ADDRESS: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
    let engine = Arc::new(Mutex::new(TransactionEngine::init()));

    let listener = TcpListener::bind(TCP_SERVER_ADDRESS).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let engine = engine.clone();
        tokio::spawn(async move {
            let mut stream = TransactionStream::from(stream);
            while let Some(transaction) = stream.next().await {
                use Transaction::*;
                let result = match transaction {
                    Deposit { client, tx, amount } => engine.lock().deposit(client, tx, amount),
                    Withdrawal { client, tx, amount } => {
                        engine.lock().withdrawal(client, tx, amount)
                    }
                    Dispute { client, tx } => engine.lock().dispute(client, tx),
                    Resolve { client, tx } => engine.lock().resolve(client, tx),
                    Chargeback { client, tx } => engine.lock().chargeback(client, tx),
                };
                if let Err(e) = result {
                    eprintln!("transaction error: {}", e);
                }
            }
        });
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
