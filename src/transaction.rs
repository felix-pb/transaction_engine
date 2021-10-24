use crate::{Amount, Balance, ClientId, Result, TransactionError};

/******************************************
 *               PUBLIC API               *
 ******************************************/

/// A unique 32-bit ID for a transaction.
pub type TransactionId = u32;

/*******************************************
 *               PRIVATE API               *
 *******************************************/

/// A single regular transaction (i.e. deposit or withdrawal).
pub(crate) struct Transaction {
    pub(crate) amount: Balance,
    pub(crate) client: ClientId,
    pub(crate) kind: TransactionKind,
    pub(crate) state: TransactionState,
}

/// A single special transaction (i.e. dispute, resolve, or chargeback).
pub(crate) struct SpecialTransaction {
    pub(crate) client: ClientId,
    pub(crate) kind: SpecialTransactionKind,
    pub(crate) tx: TransactionId,
}

pub(crate) enum TransactionKind {
    Deposit,
    Withdrawal,
}

pub(crate) enum SpecialTransactionKind {
    Dispute,
    Resolve,
    Chargeback,
}

/// A transaction state.
pub(crate) enum TransactionState {
    /// Accepted when a new regular transaction is successfully processed
    /// or when a disputed transaction is referenced by a resolve transaction.
    Accepted,
    /// Disputed when an accepted transaction is referenced by a dispute transaction.
    Disputed,
    /// Reversed when a disputed transaction is referenced by a chargeback transaction.
    /// FINAL: the state can't change at this point and the associated account is locked.
    Reversed,
}

impl Transaction {
    /// Attempts to construct a new deposit transaction.
    pub(crate) fn try_new_deposit(client: ClientId, amount: Amount) -> Result<Self> {
        Ok(Self {
            amount: try_convert_u64_to_i64(amount)?,
            client,
            kind: TransactionKind::Deposit,
            state: TransactionState::Accepted,
        })
    }

    /// Attempts to construct a new withdrawal transaction.
    pub(crate) fn try_new_withdrawal(client: ClientId, amount: Amount) -> Result<Self> {
        Ok(Self {
            amount: try_convert_u64_to_i64(amount)?,
            client,
            kind: TransactionKind::Withdrawal,
            state: TransactionState::Accepted,
        })
    }
}

impl SpecialTransaction {
    /// Constructs a new dispute transaction.
    pub(crate) fn new_dispute(client: ClientId, tx: TransactionId) -> Self {
        Self {
            client,
            kind: SpecialTransactionKind::Dispute,
            tx,
        }
    }

    /// Constructs a new resolve transaction.
    pub(crate) fn new_resolve(client: ClientId, tx: TransactionId) -> Self {
        Self {
            client,
            kind: SpecialTransactionKind::Resolve,
            tx,
        }
    }

    /// Constructs a new chargeback transaction.
    pub(crate) fn new_chargeback(client: ClientId, tx: TransactionId) -> Self {
        Self {
            client,
            kind: SpecialTransactionKind::Chargeback,
            tx,
        }
    }
}

/// Attempts to convert a `u64` to an `i64`.
///
/// Returns an `Error` if the conversion makes the amount negative (if most significant bit = 1).
fn try_convert_u64_to_i64(amount: u64) -> Result<i64> {
    let amount = amount as i64;
    if amount.is_negative() {
        Err(TransactionError::TransactionAmountTooLarge)
    } else {
        Ok(amount)
    }
}
