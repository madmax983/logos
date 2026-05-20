# 🗣️ Echo: Financial Planning examples do not compile (private module)

**🤦 The Confusion:**
I tried to run the examples in `docs/financial-planning.md` and `crates/logos-core/src/planning/mod.rs`. I copy-pasted the code directly into my `main.rs`, but the compiler gave me errors like `module 'planning' is private` and refused to compile. It said things like `use logos_core::planning::fire::FireSimulator;` are invalid. If I copy-paste the example and it doesn't compile, I am leaving.

**🕵️ The Reality:**
Turns out the `planning` module is private (`pub(crate)`) and the items are re-exported at the crate root. The examples in the documentation use the wrong import paths and instruct users to import from a private module.

**💡 The Fix:**
Update all the examples in `docs/financial-planning.md` and the rustdoc examples in `crates/logos-core/src/planning/mod.rs` to use the correct import paths (e.g., `use logos_core::fire::{FireSimulator, UpcomingVest};` instead of `use logos_core::planning::fire::{...};`).
