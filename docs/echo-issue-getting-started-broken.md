# 🗣️ Echo: Getting Started example is broken and Database connection fails

🤦 **The Confusion:**
1. I followed the `README.md` and ran the example commands. However, running `docker compose up -d db` and `sleep 3` is not enough time for Postgres to start on my machine! Running `cargo run -p logos-cli -- db migrate` immediately afterward just fails with `Error: Connection Refused: Postgres may still be starting up.`. I couldn't get the app running without manually retrying.
2. I tried copying the `logos_core` planning example into a quick script to test it out. I used `use logos_core::planning::fire::{FireSimulator, UpcomingVest};` and `use logos_core::planning::net_worth_projector::NetWorthProjector;` as shown in the docs. But it failed to compile! The compiler complained: `error[E0603]: module 'planning' is private`.

🕵️ **The Reality:**
1. Postgres 16 often takes 5 to 10 seconds to fully initialize its database files on first boot before it accepts connections. A simple `sleep 3` is unreliable.
2. The `logos-core` crate exports its planning primitives at the root using `pub use planning::*;`, but the `planning` module itself is declared as `pub(crate) mod planning;`. The examples incorrectly tell users to import from `logos_core::planning::...` instead of the direct re-exports `logos_core::fire::...` and `logos_core::net_worth_projector::NetWorthProjector`.

💡 **The Fix:**
1. Update `README.md` and any getting started scripts to wait longer (e.g., `sleep 10` or use a retry loop) or add a note about database initialization.
2. Fix the documentation examples in `crates/logos-core/src/planning/mod.rs` to use the correct, public import paths (e.g., `use logos_core::fire::{FireSimulator, UpcomingVest};` and `use logos_core::net_worth_projector::NetWorthProjector;`).
