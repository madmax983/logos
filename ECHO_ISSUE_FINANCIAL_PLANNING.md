# 🗣️ Echo: Financial Planning examples do not compile due to private modules

**🤦 The Confusion:**
I tried to run the code examples from `docs/financial-planning.md` into my fresh project's `main.rs`. When I ran `cargo run`, the compiler hit me with multiple errors saying `module 'planning' is private` and `module 'domain' is private`. I couldn't even get the basic examples to run!

**🕵️ The Reality:**
The documentation tells users to use deep import paths like `use logos_core::planning::fire::FireSimulator;`, `use logos_core::planning::net_worth_projector::NetWorthProjector;` and `use logos_core::domain::rsu::AllocationPolicy;`, but those modules are explicitly defined as `pub(crate)` in `logos-core/src/lib.rs` and are not publicly accessible to users outside the crate.

**💡 The Fix:**
Update all the examples in `docs/financial-planning.md` to use the correct, public import paths (e.g. `use logos_core::AllocationPolicy;`, `use logos_core::FireSimulator;`, `use logos_core::NetWorthProjector;`) so the code actually compiles when copy-pasted by a user.
