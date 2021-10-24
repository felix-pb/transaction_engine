use crate::{Amount, Client, ClientId, Result, TransactionError, TransactionId, TransactionState};
use crate::{SpecialTransaction, SpecialTransactionKind, Transaction, TransactionKind};
use std::collections::hash_map::{HashMap, Iter};
use TransactionError::*;
use TransactionId as Tx;

type ClientMap = HashMap<ClientId, Client>;
type TransactionMap = HashMap<Tx, Transaction>;

/******************************************
 *               PUBLIC API               *
 ******************************************/

/// A transaction engine.
pub struct TransactionEngine {
    // The `clients` field could also be implemented with a `Vec<Option<Client>>` with a fixed
    // capacity of `ClientId::MAX as usize`. The `ClientId` itself would be the index.
    // This would require 24 * 65,536 = 1,572,864 bytes of RAM on 64-bit machines but could
    // be more efficient for most operations. However, the `iter_accounts` method would probably
    // be slower to skip non-existent accounts. These are guesses and should be benchmarked.
    // Alternatively, other hash map implementations (e.g. `hashbrown::HashMap`) could be faster.
    //
    // Finally, if a transaction cannot be disputed by a different client ID, the `transactions`
    // map could be embedded in the value of the `clients` map. This would save the `ClientId`
    // storage for each transaction. Similarly, instead of storing a single `transactions` map,
    // we could split them into multiple maps to avoid storing the `TransactionKind` and
    // `TransactionState` for each transaction. In the case, only the balance would need to be
    // stored per transaction.
    clients: ClientMap,
    transactions: TransactionMap,
}

impl TransactionEngine {
    /// Constructs a new transaction engine with no history of client accounts or transactions.
    pub fn init() -> Self {
        Self {
            clients: HashMap::new(),
            transactions: HashMap::new(),
        }
    }

    /// Returns a single client account by ID.
    pub fn get_account(&self, client: ClientId) -> Option<&Client> {
        self.clients.get(&client)
    }

    /// Returns an iterator over all client accounts in arbitrary order.
    pub fn iter_accounts(&self) -> Iter<ClientId, Client> {
        self.clients.iter()
    }

    /// Attempts to process a single deposit transaction.
    pub fn deposit(&mut self, client: ClientId, tx: Tx, amount: Amount) -> Result<()> {
        let transaction = Transaction::try_new_deposit(client, amount)?;
        self.process_regular_transaction(tx, transaction)
    }

    /// Attempts to process a single withdrawal transaction.
    pub fn withdrawal(&mut self, client: ClientId, tx: Tx, amount: Amount) -> Result<()> {
        let transaction = Transaction::try_new_withdrawal(client, amount)?;
        self.process_regular_transaction(tx, transaction)
    }

    /// Attempts to process a single dispute transaction.
    pub fn dispute(&mut self, client: ClientId, tx: Tx) -> Result<()> {
        let transaction = SpecialTransaction::new_dispute(client, tx);
        self.process_special_transaction(transaction)
    }

    /// Attempts to process a single resolve transaction.
    pub fn resolve(&mut self, client: ClientId, tx: Tx) -> Result<()> {
        let transaction = SpecialTransaction::new_resolve(client, tx);
        self.process_special_transaction(transaction)
    }

    /// Attempts to process a single chargeback transaction.
    pub fn chargeback(&mut self, client: ClientId, tx: Tx) -> Result<()> {
        let transaction = SpecialTransaction::new_chargeback(client, tx);
        self.process_special_transaction(transaction)
    }
}

/*******************************************
 *               PRIVATE API               *
 *******************************************/

impl TransactionEngine {
    fn process_regular_transaction(&mut self, tx: Tx, transaction: Transaction) -> Result<()> {
        // Return an error if the transaction ID has been already processed successfully.
        if self.transactions.contains_key(&tx) {
            return Err(TransactionIdAlreadyProcessed);
        }

        // Return an error if the transaction is a withdrawal and the client account doesn't exist.
        if let TransactionKind::Withdrawal = transaction.kind {
            if !self.clients.contains_key(&transaction.client) {
                return Err(InvalidFirstTransaction);
            }
        }

        // Retrieve or create the client account. Return an error if account is locked.
        let account = retrieve_or_create_account(&mut self.clients, transaction.client)?;

        // Attempt to perform the deposit or withdrawal.
        match transaction.kind {
            TransactionKind::Deposit => account.try_deposit(transaction.amount)?,
            TransactionKind::Withdrawal => account.try_withdrawal(transaction.amount)?,
        }

        // Cache the transaction only if it was successful. Otherwise, it could be disputed.
        self.transactions.insert(tx, transaction);

        // Return successfully.
        Ok(())
    }

    fn process_special_transaction(&mut self, transaction: SpecialTransaction) -> Result<()> {
        // Return an error if the transaction ID hasn't been already processed successfully.
        let old_transaction = self
            .transactions
            .get_mut(&transaction.tx)
            .ok_or(UnknownTransactionId)?;

        // Return an error if this transaction's client ID doesn't match the old one.
        if transaction.client != old_transaction.client {
            return Err(WrongClientId);
        }

        // Retrieve or create the client account. Return an error if account is locked.
        let account = retrieve_or_create_account(&mut self.clients, transaction.client)?;

        // Attempt to perform the dispute, resolve, or chargeback.
        match transaction.kind {
            SpecialTransactionKind::Dispute => match old_transaction.state {
                TransactionState::Accepted => account.try_dispute(old_transaction)?,
                TransactionState::Disputed => return Err(TransactionAlreadyDisputed),
                TransactionState::Reversed => unreachable!("account should be locked"),
            },
            SpecialTransactionKind::Resolve => match old_transaction.state {
                TransactionState::Accepted => return Err(TransactionNotDisputed),
                TransactionState::Disputed => account.try_resolve(old_transaction)?,
                TransactionState::Reversed => unreachable!("account should be locked"),
            },
            SpecialTransactionKind::Chargeback => match old_transaction.state {
                TransactionState::Accepted => return Err(TransactionNotDisputed),
                TransactionState::Disputed => account.try_chargeback(old_transaction)?,
                TransactionState::Reversed => unreachable!("account should be locked"),
            },
        }

        // Return successfully.
        Ok(())
    }
}

/// Retrieves a client account by ID, or creates a new account if it doesn't already exist.
///
/// Returns an `Error` if the client account is locked.
fn retrieve_or_create_account(clients: &mut ClientMap, client: ClientId) -> Result<&mut Client> {
    let account = clients.entry(client).or_insert_with(Client::init);
    if account.is_locked() {
        return Err(ClientAccountLocked);
    }
    Ok(account)
}
