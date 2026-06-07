# 🗣️ Echo: Financial Planning docs examples do not compile

**🤦 The Confusion:**
Tried to run the `fire` and `net_worth_projector` examples from `docs/financial-planning.md` and `crates/logos-core/src/planning/mod.rs`. The compiler threw an `E0603` error because the `planning` module is private, but the examples suggest using `use logos_core::planning::fire::{FireSimulator, UpcomingVest};`. Also, what is a 'haircut-adjusted net worth'?

**🕵️ The Reality:**
The `planning` module is declared as `pub(crate) mod planning;` but its contents are re-exported using `pub use planning::*;`. So the correct import path is `logos_core::fire::{...}` instead of `logos_core::planning::fire::{...}`. The 'haircut' term is jargon for a discount or reduction in value, but new users might not know this.

**💡 The Fix:**
Update the documentation examples in `docs/financial-planning.md` and `crates/logos-core/src/planning/mod.rs` to use the correct `logos_core::fire` import path. Replace or explain the 'haircut' jargon.
