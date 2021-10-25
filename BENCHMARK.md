## Benchmark

I've included a simple benchmark in `examples/benchmark.rs` that does the following:
- Process 65.536 million transactions by going through each client account 1000 times in a round-robin manner.
- Make deposits for the first half and withdrawals for the second half.

On my macOS laptop, it runs in ~17.8 seconds in release mode.

Here are the update benchmark results when making the following changes in isolation:
- ~15.8 seconds when both instances of `std::collections::HashMap` have their capacity allocated ahead of time.
- ~12.6 seconds when both instances of `std::collections::HashMap` are replaced with `hashbrown::HashMap`.
- ~10.5 seconds when both instances of `hashbrown::HashMap` have their capacity allocated ahead of time.
* TODO: Vec(Option(T))

Note that removing the overflow checks in `client.rs` and `transaction.rs` doesn't have a noticeable impact on the benchmark results. That's great because these checks are necessary for the transaction to run correctly under any scenario!
