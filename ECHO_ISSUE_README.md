# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
I tried to run the tool by copy-pasting the exact code block from `docs/financial-planning.md` into a new project's `src/main.rs`. When I tried to compile it, it completely failed. It said `planning` is a private module and couldn't find `logos_core`. I assumed it was completely broken.

**🕵️ The Reality:**
The code example instructs to import `logos_core::planning::fire::{FireSimulator, UpcomingVest}` and `logos_core::planning::net_worth_projector::NetWorthProjector`. However, `planning` is a private module (`pub(crate) mod planning`). The types are re-exported at the crate root, so the correct imports are `logos_core::fire::{...}` and `logos_core::net_worth_projector::{...}`.

**💡 The Fix:**
Fix the code examples in `docs/financial-planning.md` to use the correct root imports rather than trying to access the private `planning` module directly.
