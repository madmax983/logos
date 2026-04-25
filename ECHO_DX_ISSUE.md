# 🗣️ Echo: Getting Started and Examples DX Issues

**🤦 The Confusion:**
1. I tried to run `run_readme.sh` to execute the examples from the README. The script hung indefinitely on `cargo run -p logos-tui` and timed out because it launched an interactive terminal UI in a non-interactive bash script, preventing the process from exiting.
2. I tried to run the script `run_echo_test.sh` to test out the NetWorthProjector examples, and it crashed with `src/main.rs: No such file or directory`. The script tried to write to `src/main.rs` but the current working directory didn't have a `src/` folder. It also tried to run `cargo run` outside of a proper cargo workspace/binary package structure.
3. In `docs/financial-planning.md`, the `RsuAutoDistributor` example crashes with a compilation error if copy-pasted into a new crate because `AllocationPolicy::new(40, 20, 30, 10)` expects `u8` arguments, but the example fails to mention that `logos-core` might have required features (like `nova`) or that `TransactionBuilder` isn't imported. Similarly, testing `AllocationPolicy` with an invalid total (e.g. 101) outputs `InvalidAllocationTotal { total: 101 }` which is just a raw derived Debug string instead of a friendly human-readable error message in standard Display format.

**🕵️ The Reality:**
1. `run_readme.sh` blindly copy-pastes the entire README code block, including the interactive TUI command `cargo run -p logos-tui`.
2. `run_echo_test.sh` attempts to write `src/main.rs` directly in the repository root without creating a temporary crate (like `cargo new`) first, so the `src/` directory is missing.
3. For errors in `logos-core`, derived `Debug` traits are frequently exposed directly to users instead of helpful `Display` formats. Experimental features often require `#![cfg(feature = "nova")]` or similar flags which aren't clearly documented in the inline examples.

**💡 The Fix:**
1. Remove or comment out the interactive `cargo run -p logos-tui` command from `run_readme.sh` so it can be executed headless, or modify the README block to denote that `logos-tui` is interactive.
2. Update `run_echo_test.sh` to use `cargo new echo_test && cd echo_test && cargo add --path ../crates/logos-core` before writing to `src/main.rs`, and run `cargo run` from within that directory.
3. Ensure all domain primitives implement standard `fmt::Display` so error messages are human-readable, and verify that `docs/financial-planning.md` examples compile perfectly in a fresh crate context with all necessary imports included.
