# transaction_engine

A transaction engine to process deposits, withdrawals, disputes, resolves, and chargebacks.

## Usage

Minimum supported rust version (MSRV): 1.56.0

Run this command to run the transaction engine with the given input CSV file:
```
cargo run -- <input csv file path>
```

The final state of client accounts is printed to stdout, while warning and error messages are printed to stderr.

## Project structure

This project provides 1 binary crate and 1 library crate.

- The binary crate's code is fully contained in `main.rs` (excluding dependencies).
- The library crate's code consists of all other files in `src/`.

## Library

The library crate provides the core implementation of the transaction engine. This is the focus of this project, and therefore the code is clean, modular, and well documented.

Run this command to check the documentation for the public API:
```
cargo doc --open
```

## Binary

The binary crate is just a simple wrapper around the library to handle CSV input deserialization and output serialization. It should be functionally correct, although less care was taken to write clean and modular code. That's because, in a real-world scenario, the transaction engine would probably be bundled with a server (instead of a CSV file reader). The library implementation should make it trivial to do so (e.g. with Tokio) and to process transactions in a "streaming" manner.

## Tests

All integration tests are based around the simple `test_csv!` macro in `tests/integration_tests.rs`. This macro runs the binary crate with an input CSV file and compares the actual output with an expected output CSV file.

Steps to add an integration test:

1. Choose a meaningful name for your test.
2. Create an input CSV file with that name in `tests/input/`.
3. Create an expected output CSV file with that name in `tests/output/`.
4. Add a `test_csv!(<name>)` line in `tests/integration_tests.rs`.

There are no unit tests currently, although it would be worthwhile to add some given more time.

## Assumptions

- A dispute, resolve, or chargeback is only accepted if its client ID matches the client ID of the original referenced transaction. If that's not necessary, the current implementation can support this simply by commenting out 3 lines. However, if it's indeed a requirement, the implementation could be optimized for it. Check `src/engine.rs` for suggestions of alternative internal data structures.
- Both deposits and withdrawals can be disputed. For a deposit, the associated funds are held and the available balance is decreased accordingly. For a withdrawal, the associated funds are NOT held and the available balance stays the same. If a withdrawal is reversed, the associated funds are deposited back into the account (i.e. the available and total balances both increase by that amount).
