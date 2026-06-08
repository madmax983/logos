# 🗣️ Echo: Financial Planning docs use private module paths

**🤦 The Confusion:**
I wanted to try out the new financial planning primitives. I copied the code examples from `docs/financial-planning.md` into my own project and ran `cargo build`. It instantly failed with multiple `error[E0603]: module 'planning' is private` messages when trying to import from `logos_core::planning::fire`, `logos_core::planning::net_worth_projector`, etc. I thought the features weren't ready for public use yet.

**🕵️ The Reality:**
The `planning` module inside `logos-core` is crate-private (`pub(crate) mod planning;`), but its contents are re-exported at the crate root (`pub use planning::*;`). The examples in the documentation use the internal, private paths instead of the public, re-exported paths.

**💡 The Fix:**
Update all code examples in `docs/financial-planning.md` to use the correct public import paths. For example, change `use logos_core::planning::fire::FireSimulator;` to `use logos_core::fire::FireSimulator;` and `use logos_core::planning::net_worth_projector::NetWorthProjector;` to `use logos_core::net_worth_projector::NetWorthProjector;`.
