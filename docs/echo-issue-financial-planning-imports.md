# 🗣️ Echo: Financial Planning examples do not compile (private modules)

**🤦 The Confusion:**
I was trying to learn how to use the new financial planning tools. I copied the example code that sets up `FireSimulator` and `NetWorthProjector` to see how my future net worth looks. But when I ran `cargo check`, the compiler yelled at me! It said `error[E0603]: module 'planning' is private` for `logos_core::planning::fire::{FireSimulator, UpcomingVest}`. I literally copy-pasted the example, and it's completely broken because I'm not allowed to access those modules.

**🕵️ The Reality:**
Turns out `logos-core` uses a Facade pattern where `planning` is actually an internal `pub(crate)` module. The library exports the types at the root level (like `logos_core::fire` instead of `logos_core::planning::fire`), but the documentation still uses the internal paths.

**💡 The Fix:**
Update all the `use` statements in the examples to use the public root-level re-exports (e.g., `use logos_core::fire::{FireSimulator, UpcomingVest};` and `use logos_core::net_worth_projector::NetWorthProjector;`). Please fix the docs!
