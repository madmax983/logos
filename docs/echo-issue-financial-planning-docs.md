# 🗣️ Echo: Financial Planning examples do not compile

🤦 **The Confusion:**
I tried to follow the examples in `docs/financial-planning.md` to use the `FireSimulator` and `NetWorthProjector`. I copy-pasted the code into my `main.rs`, but `cargo run` exploded with errors saying `module 'planning' is private`.

🕵️ **The Reality:**
I looked at the compiler error. The docs tell me to use `logos_core::planning::fire::{FireSimulator, UpcomingVest}` and `logos_core::planning::net_worth_projector::NetWorthProjector`, but the `planning` module is private! I shouldn't have to read the source code to figure out what to import.

💡 **The Fix:**
The code blocks in the documentation need to be updated to use the correct public export paths, e.g., `logos_core::fire::{FireSimulator, UpcomingVest}` and `logos_core::net_worth_projector::NetWorthProjector`.
