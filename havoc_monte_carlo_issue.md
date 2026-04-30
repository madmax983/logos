Title: "👺 Havoc: `MonteCarloProjector` Panics on Memory Allocation Overflow"

🧨 **The Trigger:**
Passing `u32::MAX` as the number of paths to `MonteCarloProjector::run` causes a massive memory allocation request via `Vec::with_capacity(paths as usize)`, resulting in a panic due to capacity overflow.

📉 **The Stack Trace:**
```
memory allocation of 34359738360 bytes failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: test failed, to rerun pass `-p logos-core --test havoc_monte_carlo`

Caused by:
  process didn't exit successfully: `/app/target/debug/deps/havoc_monte_carlo-8031619532457a0a havoc_monte_carlo` (signal: 6, SIGABRT: process abort signal)
```

🧪 **Reproduction:**
Run `cargo test -p logos-core havoc_monte_carlo --features nova`.

😈 **Comment:**
"You assumed Monte Carlo simulations would only run with a reasonable number of paths. You were wrong. Passing `u32::MAX` completely crashes the simulator by trying to allocate more RAM than the system has instead of returning an error."
