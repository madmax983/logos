# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
I was reading `docs/financial-planning.md` and tried to use the new `RsuAutoDistributor`, `FireSimulator`, and `NetWorthProjector` components. When I copy-pasted the example code directly into my `main.rs`, the compiler threw multiple `[E0603]: module '...' is private` errors. I couldn't even use the core features because the examples in the docs wouldn't compile!

**🕵️ The Reality:**
The example code uses nested private module paths like `logos_core::domain::account::AccountId` and `logos_core::planning::fire::FireSimulator`. However, the `domain` and `planning` modules are marked as private (`pub(crate) mod domain` and `pub(crate) mod planning`). The examples in the docs do not reflect the actual public API paths as suggested by the compiler.

**💡 The Fix:**
Update the examples in `docs/financial-planning.md` and the module documentation in `crates/logos-core/src/planning/mod.rs` to use the correct public import paths suggested by the compiler. For instance, change `use logos_core::planning::fire::FireSimulator` to `use logos_core::fire::FireSimulator`, and change `use logos_core::planning::net_worth_projector::NetWorthProjector` to `use logos_core::net_worth_projector::NetWorthProjector`.