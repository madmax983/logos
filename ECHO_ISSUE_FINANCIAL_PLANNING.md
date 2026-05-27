# 🗣️ Echo: Financial Planning examples are broken

**🤦 The Confusion:**
I wanted to try the new planning features, so I copied the exact code examples from `docs/financial-planning.md` into my `main.rs` file. When I ran `cargo run`, it immediately failed to compile with a bunch of `error[E0603]: module 'planning' is private` messages. I thought the library was completely broken.

**🕵️ The Reality:**
The `planning` module in `logos-core` is marked as `pub(crate)` and its components are re-exported at the crate root. The documentation in `docs/financial-planning.md` incorrectly instructs users to import from `logos_core::planning::...` instead of just `logos_core::...`.

**💡 The Fix:**
Update the documentation in `docs/financial-planning.md` to use the correct import paths (e.g., `use logos_core::fire::FireSimulator;` instead of `use logos_core::planning::fire::FireSimulator;`).
