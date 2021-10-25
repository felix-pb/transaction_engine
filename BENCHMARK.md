## Benchmark

I've included a simple benchmark in `examples/benchmark.rs` that does the following:
- Process 65.536 million transactions by going through each client account 1000 times in a round-robin manner.
- Make deposits for the first half and withdrawals for the second half.

On my macOS laptop, it runs in ~17.8 seconds in release mode.

* TODO: no overflow checks
* TODO: hashbrown::HashMap
* TODO: Vec(Option(T))
