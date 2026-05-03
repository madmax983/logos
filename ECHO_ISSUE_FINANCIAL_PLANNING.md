# 🗣️ Echo: Financial Planning docs have broken imports

**🤦 The Confusion:**
I wanted to try the new financial planning features by copying the examples directly from `docs/financial-planning.md`. I pasted the `RsuAutoDistributor` and `FireSimulator` examples into a fresh file and tried to run them. The compiler immediately threw errors complaining about private modules, saying things like `module domain is private` and `module planning is private`. I couldn't even get the basic examples to compile!

**🕵️ The Reality:**
The documentation tells me to use paths like `logos_core::domain::account::AccountId` and `logos_core::planning::fire::FireSimulator`. However, these internal modules are actually marked `pub(crate)` and the types are re-exported at the root or top-level public modules of `logos_core`. The examples in the docs are pointing to the wrong internal paths.

**💡 The Fix:**
Update all the examples in `docs/financial-planning.md` to use the correct public re-exports (e.g., `use logos_core::AccountId;` and `use logos_core::fire::FireSimulator;`) instead of exposing the private internal `domain` and `planning` module paths.
