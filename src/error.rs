use thiserror::Error;

/******************************************
 *               PUBLIC API               *
 ******************************************/

/// A transaction error.
///
/// When that happens, the transaction engine must guarantee that
/// the client account referenced by this transaction was NOT modified.
#[derive(Debug, Error)]
pub enum TransactionError {
    #[error("balance would overflow")]
    BalanceWouldOverflow,
    #[error("client account locked")]
    ClientAccountLocked,
    #[error("insufficient available funds")]
    InsufficientAvailableFunds,
    #[error("invalid first transaction")]
    InvalidFirstTransaction,
    #[error("transaction already disputed")]
    TransactionAlreadyDisputed,
    #[error("transaction amount too large")]
    TransactionAmountTooLarge,
    #[error("transaction id already processed")]
    TransactionIdAlreadyProcessed,
    #[error("transaction not disputed")]
    TransactionNotDisputed,
    #[error("unknown transaction id")]
    UnknownTransactionId,
    #[error("wrong client id")]
    WrongClientId,
}
