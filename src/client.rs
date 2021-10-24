use crate::{Balance, Result, Transaction, TransactionError, TransactionKind, TransactionState};

/******************************************
 *               PUBLIC API               *
 ******************************************/

/// A unique 16-bit ID for a client account.
pub type ClientId = u16;

/// A client account.
pub struct Client {
    available: Balance,
    total: Balance,
    locked: bool,
}

impl Client {
    /// Returns the available balance in this client account.
    pub fn get_available_balance(&self) -> Balance {
        self.available
    }

    /// Returns the held balance in this client account.
    pub fn get_held_balance(&self) -> Balance {
        // The transaction engine *must* guarantee that this method does NOT overflow.
        // The implementation *should* guarantee that this unwrap won't panic.
        // However, if there's a bug, better death than dishonor.
        self.total.checked_sub(self.available).unwrap()
    }

    /// Returns the total balance in this client account.
    pub fn get_total_balance(&self) -> Balance {
        self.total
    }

    /// Returns whether this client account is locked.
    pub fn is_locked(&self) -> bool {
        self.locked
    }
}

/*******************************************
 *               PRIVATE API               *
 *******************************************/

impl Client {
    /// Constructs a new client account with a balance of zero.
    pub(crate) fn init() -> Self {
        Self {
            available: 0,
            total: 0,
            locked: false,
        }
    }

    /// Attempts to process a deposit in this client account.
    pub(crate) fn try_deposit(&mut self, amount: Balance) -> Result<()> {
        self.total = checked_add_balance(self.total, amount)?;
        // The transaction engine *must* guarantee that this method does NOT overflow.
        // The implementation *should* guarantee that this unwrap won't panic.
        // However, if there's a bug, better death than dishonor.
        self.available = checked_add_balance(self.available, amount).unwrap();
        Ok(())
    }

    /// Attempts to process a withdrawal in this client account.
    pub(crate) fn try_withdrawal(&mut self, amount: Balance) -> Result<()> {
        if amount > self.available {
            return Err(TransactionError::InsufficientAvailableFunds);
        }
        // The transaction engine *must* guarantee that this method does NOT overflow.
        // The implementation *should* guarantee that these unwraps won't panic.
        // However, if there's a bug, better death than dishonor.
        self.available = checked_sub_balance(self.available, amount).unwrap();
        self.total = checked_sub_balance(self.total, amount).unwrap();
        Ok(())
    }

    /// Attempts to process a dispute in this client account.
    pub(crate) fn try_dispute(&mut self, old_transaction: &mut Transaction) -> Result<()> {
        if let TransactionKind::Deposit = old_transaction.kind {
            self.available = checked_sub_balance(self.available, old_transaction.amount)?;
        }
        old_transaction.state = TransactionState::Disputed;
        Ok(())
    }

    /// Attempts to process a resolve in this client account.
    pub(crate) fn try_resolve(&mut self, old_transaction: &mut Transaction) -> Result<()> {
        if let TransactionKind::Deposit = old_transaction.kind {
            self.available = checked_add_balance(self.available, old_transaction.amount)?;
        }
        old_transaction.state = TransactionState::Accepted;
        Ok(())
    }

    /// Attempts to process a chargeback in this client account.
    pub(crate) fn try_chargeback(&mut self, old_transaction: &mut Transaction) -> Result<()> {
        let amount = old_transaction.amount;
        match old_transaction.kind {
            TransactionKind::Deposit => self.total = checked_sub_balance(self.total, amount)?,
            TransactionKind::Withdrawal => self.try_deposit(amount)?,
        }
        old_transaction.state = TransactionState::Reversed;
        self.locked = true;
        Ok(())
    }
}

/// Attempts to add `amount` to `old_balance`.
///
/// Returns an `Error` if overflow would occur.
fn checked_add_balance(old_balance: Balance, amount: Balance) -> Result<Balance> {
    old_balance
        .checked_add(amount)
        .ok_or(TransactionError::BalanceWouldOverflow)
}

/// Attempts to subtract `amount` from `old_balance`.
///
/// Returns an `Error` if overflow would occur.
fn checked_sub_balance(old_balance: Balance, amount: Balance) -> Result<Balance> {
    old_balance
        .checked_sub(amount)
        .ok_or(TransactionError::BalanceWouldOverflow)
}
