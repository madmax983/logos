# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
Tried to run the `story_demo` or copy the example in `README.md` and `docs/financial-planning.md`. Compiler said `module `planning` is private`.

**🕵️ The Reality:**
Turns out `logos-core::planning` is private. I need to import directly from `logos_core` (e.g. `use logos_core::fire::{FireSimulator, UpcomingVest};`).

**💡 The Fix:**
Update the examples in `README.md`, `crates/logos-core/src/planning/mod.rs` and `docs/financial-planning.md` to use the correct imports (e.g. `use logos_core::fire::{FireSimulator, UpcomingVest};`).
