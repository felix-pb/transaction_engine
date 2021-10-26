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

Run this command to run all the tests:
```
cargo test
```

There are no unit tests currently, although it would be worthwhile to add some given more time.

## Assumptions

- A dispute, resolve, or chargeback is only accepted if its client ID matches the client ID of the original referenced transaction. If that's not necessary, the current implementation can support this simply by commenting out 3 lines. However, if it's indeed a requirement, the implementation could be optimized for it. Check `src/engine.rs` for suggestions of alternative internal data structures.
- Both deposits and withdrawals can be disputed. For a deposit, the associated funds are held and the available balance is decreased accordingly. For a withdrawal, the associated funds are NOT held and the available balance stays the same. If a withdrawal is reversed, the associated funds are deposited back into the account (i.e. the available and total balances both increase by that amount).

## Benchmark

I've included a simple benchmark in `examples/single-threaded.rs` that does the following:
- Process 65.536 million transactions by going through each client account 1000 times in a round-robin manner.
- Make deposits for the first half and withdrawals for the second half.

On my macOS laptop, it runs in ~17.8 seconds in release mode.

By making the following changes in isolation, I get the following updated results:
- ~15.8 seconds when both hash maps have their capacity allocated ahead of time.
- ~12.6 seconds when both hash maps are replaced with `hashbrown::HashMap`.
- ~10.5 seconds when both hash maps are replaced with `hashbrown::HashMap` and have their capacity allocated ahead of time.
- ~396 milliseconds when both hash maps are replaced with `Vec<Option<T>>` and have their capacity allocated ahead of time with `None` values.

Although not surprising, the performance improvement for the last change is so big that it's worth discussing it more. In particular, it's worth discussing the tradeoff in terms of memory usage:
- The size of an `Option<Client>` is 24 bytes. The maximum number of clients is 65,536 (i.e. `u16::MAX` + 1). Therefore, the total storage requirement is ~1.6 MB for that vector. That's pretty reasonable so I won't discuss it further.
- The size of an `Option<Transaction>` is 16 bytes. In the benchmark above, I was processing 65.536 million transactions and therefore needed a total storage requirement of ~1.0 GB for that vector. However, the maximum number of transactions is 4,294,967,296 (i.e. `u32::MAX` + 1). Therefore, without making any change to the current `Transaction` struct, the total storage requirement would be ~68.7 GB. This is not an unreasonable amount of RAM for modern servers. That said, it would be possible to optimize this data structure for space if needed (at the cost of less elegant code).

Note that removing the overflow checks in `client.rs` and `transaction.rs` doesn't have a noticeable impact on the benchmark results. That's great because these checks are necessary for the transaction engine to run correctly under any scenario!
