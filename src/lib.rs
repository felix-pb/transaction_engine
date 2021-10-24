//! This library crate provides a transaction engine
//! to process deposits, withdrawals, disputes, resolves, and chargebacks.
//!
//! ## Example
//! ```
//! use transaction_engine::TransactionEngine;
//!
//! // Initialize the transaction engine
//! let mut engine = TransactionEngine::init();
//!
//! // Process 5 transactions...
//! // #1: deposit 1.0000 to 1st client's account
//! // #2: deposit 2.0000 to 2nd client's account
//! // #3: deposit 2.0000 to 1st client's account
//! // #4: withdraw 1.5000 from 1st client's account
//! // #5: withdraw 3.0000 from 2nd client's account --> ERROR!
//! engine.deposit(1, 1, 1_0000).unwrap();
//! engine.deposit(2, 2, 2_0000).unwrap();
//! engine.deposit(1, 3, 2_0000).unwrap();
//! engine.withdrawal(1, 4, 1_5000).unwrap();
//! engine.withdrawal(2, 5, 3_0000).unwrap_err();
//!
//! // Validate 1st client's account
//! let client1 = engine.get_account(1).unwrap();
//! assert_eq!(1_5000, client1.get_available_balance());
//! assert_eq!(0_0000, client1.get_held_balance());
//! assert_eq!(1_5000, client1.get_total_balance());
//! assert!(!client1.is_locked());
//!
//! // Validate 2nd client's account
//! let client2 = engine.get_account(2).unwrap();
//! assert_eq!(2_0000, client2.get_available_balance());
//! assert_eq!(0_0000, client2.get_held_balance());
//! assert_eq!(2_0000, client2.get_total_balance());
//! assert!(!client2.is_locked());
//! ```

mod client;
mod engine;
mod error;
mod transaction;

/******************************************
 *               PUBLIC API               *
 ******************************************/

pub use client::{Client, ClientId};
pub use engine::TransactionEngine;
pub use error::TransactionError;
pub use transaction::TransactionId;

/// An amount in the smallest unit of currency.
pub type Amount = u64;

/// An account balance in the smallest unit of currency.
///
/// This is an `i64` instead of a `u64` because an account balance can become negative
/// if a deposit is successfully disputed after being withdrawn.
pub type Balance = i64;

/*******************************************
 *               PRIVATE API               *
 *******************************************/

pub(crate) use transaction::*;

/// A transaction result.
pub(crate) type Result<T> = std::result::Result<T, TransactionError>;
