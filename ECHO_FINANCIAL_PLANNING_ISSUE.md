# 🗣️ Echo: Financial Planning docs examples are broken

🤦 **The Confusion:**
I tried to follow the examples in `docs/financial-planning.md` to run the financial planning engine. I copied and pasted the examples for `FireSimulator`, `NetWorthProjector`, and `RsuAutoDistributor` exactly as they were written. But when I ran `cargo check`, it failed with `error[E0603]: module 'planning' is private`! The compiler refused to let me import `logos_core::planning::fire::FireSimulator` and the others.

🕵️ **The Reality:**
Turns out the `planning` module inside `logos_core` is declared as `pub(crate) mod planning;` in `crates/logos-core/src/lib.rs`. The examples in the docs show `use logos_core::planning::fire::FireSimulator;`, which fails. The correct way to import them is directly from the re-exported modules at the crate root, like `use logos_core::fire::FireSimulator;` and `use logos_core::net_worth_projector::NetWorthProjector;`.

💡 **The Fix:**
Update the code examples in `docs/financial-planning.md` to use the correct import paths from the crate root (e.g., `use logos_core::fire::{FireSimulator, UpcomingVest};` instead of `use logos_core::planning::fire::{FireSimulator, UpcomingVest};`).
