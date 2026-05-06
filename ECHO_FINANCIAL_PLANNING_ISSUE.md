# 🗣️ Echo: Financial Planning examples in docs/financial-planning.md are broken

**🤦 The Confusion:**
I tried to run the examples in `docs/financial-planning.md` by copying them into my code. The examples failed to compile with a bunch of `E0603` errors saying that `module 'planning' is private` when I tried to use `logos_core::planning::fire::FireSimulator`, `logos_core::planning::rsu_distributor::RsuAutoDistributor`, and `logos_core::planning::net_worth_projector::NetWorthProjector`.

**🕵️ The Reality:**
Turns out the `planning` module is marked `pub(crate)` in `logos_core`, but its contents are re-exported at the crate root (`pub use planning::*;`). So the documentation is telling users to import from a private path instead of the public root re-exports.

**💡 The Fix:**
Update the examples in `docs/financial-planning.md` to use the root re-exports (e.g., `use logos_core::fire::FireSimulator;` instead of `use logos_core::planning::fire::FireSimulator;`).