# 🗣️ Echo: Financial Planning docs have broken imports

🤦 **The Confusion:**
I tried to copy-paste the code examples from `docs/financial-planning.md` to try out the FIRE Simulator and Net Worth Projector. When I compiled it, I got a bunch of errors saying `error[E0603]: module 'planning' is private`! Imports like `use logos_core::planning::fire::{FireSimulator, UpcomingVest};` did not work.

🕵️ **The Reality:**
It turns out the `planning` module is marked `pub(crate)` in the `logos_core` library, so external users can't access it directly. The types are actually re-exported at the crate root, so we should be importing them as `logos_core::fire`, `logos_core::net_worth_projector`, etc., without the `planning` prefix.

💡 **The Fix:**
Update `docs/financial-planning.md` and any other similar examples to use the correct root imports (e.g., `use logos_core::fire::{FireSimulator, UpcomingVest};`) so the examples actually compile when users copy them.
