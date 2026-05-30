# 🗣️ Echo: Getting Started example for financial planning is broken

**🤦 The Confusion:**
I wanted to try the financial planning primitives by copying the example blocks from the `docs/financial-planning.md`. I pasted the example into a fresh `src/main.rs` file and ran `cargo run`. The compiler crashed with multiple `error[E0603]: module \`planning\` is private` errors. It said `use logos_core::planning::fire::{FireSimulator, UpcomingVest};` is using a private module.

**🕵️ The Reality:**
The `logos-core` crate exposes the `planning` module internally as `pub(crate) mod planning;`. It re-exports its contents at the root level, making them accessible via `logos_core::fire` and `logos_core::net_worth_projector`. This means the `planning` namespace is completely hidden from external users, yet all the documentation examples incorrectly tell users to import from `logos_core::planning::*`. When users follow the docs exactly, the code fails to compile.

**💡 The Fix:**
Update all documentation examples (in `docs/financial-planning.md` and intra-doc comments) to use the correct top-level public re-exports (e.g., `use logos_core::fire::FireSimulator;` or `use logos_core::net_worth_projector::NetWorthProjector;`), or make the `planning` module fully public so the example paths actually work.
