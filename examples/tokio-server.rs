use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use transaction_engine::{Amount, ClientId, TransactionEngine, TransactionId};

const TCP_SERVER_ADDRESS: &str = "127.0.0.1:8080";
const TRANSACTION_BUFFER_SIZE: usize = 10_000;

#[tokio::main]
async fn main() {
    let mut engine = TransactionEngine::init();
    let (sender, mut receiver) = tokio::sync::mpsc::channel(TRANSACTION_BUFFER_SIZE);

    // For max performance, pin this thread to an isolated core.
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
    });

    let listener = TcpListener::bind(TCP_SERVER_ADDRESS).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        let sender = sender.clone();
        tokio::spawn(async move {
            let mut stream = TransactionStream::from(stream);
            while let Some(transaction) = stream.next().await {
                sender.send(transaction).await.unwrap();
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
        self.stream.read_exact(&mut self.buffer).await.unwrap();
        // Do stuff...
        Some(Transaction::Deposit {
            client: 1,
            tx: 1,
            amount: 1,
        })
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
