use csv::{ReaderBuilder, Trim, Writer};
use rust_decimal::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use transaction_engine::{ClientId, TransactionEngine, TransactionId};

/*****************************************
 *               CSV INPUT               *
 *****************************************/

#[derive(Deserialize)]
struct InputCsvRow {
    r#type: TransactionType,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Amount>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

struct Amount(u64);

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let string = String::deserialize(deserializer)?;
        let decimal = Decimal::from_str(&string).map_err(Error::custom)?;
        let decimal = match decimal.checked_mul(Decimal::new(10000, 0)) {
            Some(amount) => amount,
            None => return Err(Error::custom("invalid amount")),
        };
        let amount = match decimal.to_u64() {
            Some(amount) => amount,
            None => return Err(Error::custom("invalid amount")),
        };
        Ok(Amount(amount))
    }
}

/******************************************
 *               CSV OUTPUT               *
 ******************************************/

#[derive(Serialize)]
struct OutputCsvRow {
    client: ClientId,
    available: Balance,
    held: Balance,
    total: Balance,
    locked: bool,
}

struct Balance(i64);

impl Serialize for Balance {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let decimal = Decimal::new(self.0, 4);
        serializer.serialize_str(&decimal.to_string())
    }
}

/************************************
 *               MAIN               *
 ************************************/

fn main() {
    // Read the input CSV file's path from the first argument.
    // On failure, print an error message and exit the program.
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("error: please provide path to input csv file as first argument");
        std::process::exit(1);
    });

    // Build a CSV file reader that trims leading and trailing whitespaces.
    // On failure, print an error message and exit the program.
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(&path)
        .unwrap_or_else(|e| {
            eprintln!("error: failed to read {:?}: {}", path, e);
            std::process::exit(1);
        });

    // Initialize the transaction engine.
    let mut engine = TransactionEngine::init();

    // Deserialize the input CSV file row by row...
    for row_result in reader.deserialize() {
        // Parse the current row into a `InputCsvRow` struct.
        // On failure, print a warning message and continue to next row.
        let row: InputCsvRow = match row_result {
            Ok(row) => row,
            Err(e) => {
                eprintln!("warning: failed to parse row: {}", e);
                continue;
            }
        };

        // Attempt to process the transaction.
        let transaction_result = match row.amount {
            Some(amount) => match row.r#type {
                TransactionType::Deposit => engine.deposit(row.client, row.tx, amount.0),
                TransactionType::Withdrawal => engine.withdrawal(row.client, row.tx, amount.0),
                _ => {
                    eprintln!("warning: {:?} cannot specify amount", row.r#type);
                    continue;
                }
            },
            None => match row.r#type {
                TransactionType::Dispute => engine.dispute(row.client, row.tx),
                TransactionType::Resolve => engine.resolve(row.client, row.tx),
                TransactionType::Chargeback => engine.chargeback(row.client, row.tx),
                _ => {
                    eprintln!("warning: {:?} must specify amount", row.r#type);
                    continue;
                }
            },
        };

        // If the transaction returned an error, print a warning and continue to next row.
        if let Err(e) = transaction_result {
            eprintln!("warning: transaction failed: {}", e);
        }
    }

    // At this point, all input rows (i.e. transactions) have been processed.
    // Collect all client accounts into a vector of `OutputCsvRow` structs.
    let mut rows = engine
        .iter_accounts()
        .map(|(id, account)| OutputCsvRow {
            client: id,
            available: Balance(account.get_available_balance()),
            held: Balance(account.get_held_balance()),
            total: Balance(account.get_total_balance()),
            locked: account.is_locked(),
        })
        .collect::<Vec<_>>();

    // Sort the rows by client ID to generate deterministic output.
    // This is required for the integration tests to work.
    rows.sort_by_key(|k| k.client);

    // Write all the rows to standard output.
    let mut writer = Writer::from_writer(std::io::stdout());
    rows.iter().for_each(|row| writer.serialize(row).unwrap());
}
