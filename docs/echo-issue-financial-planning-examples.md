# 🗣️ Echo: Financial Planning docs examples fail to compile

**🤦 The Confusion:**
I wanted to try out the new `RsuAutoDistributor` and `FireSimulator` features, so I copied the example code snippets straight from `docs/financial-planning.md` into a new project. When I tried to compile, it immediately blew up with errors like `module 'domain' is private` and `module 'planning' is private`. I thought I was using the wrong version of the library or that I needed to enable some hidden feature flag.

**🕵️ The Reality:**
I did the "README Run" by pasting the exact blocks into a new binary targeting the `logos-core` crate. The code examples use internal paths like `logos_core::domain::account::AccountId` and `logos_core::planning::fire::FireSimulator`. However, in `logos-core/src/lib.rs`, `domain` and `planning` are marked as `pub(crate) mod`, making them private from the outside. The library actually re-exports the types directly at the root (e.g., `pub use domain::account::AccountId;` and `pub use planning::*;`), so the paths in the examples are structurally impossible to use from an external crate.

**💡 The Fix:**
Update the code snippets in `docs/financial-planning.md` to use the correct public re-exports. Change `logos_core::domain::account::AccountId` to `logos_core::AccountId`, change `logos_core::domain::rsu::AllocationPolicy` to `logos_core::AllocationPolicy`, change `logos_core::planning::rsu_distributor::...` to `logos_core::rsu_distributor::...`, change `logos_core::planning::fire::...` to `logos_core::fire::...`, and change `logos_core::planning::net_worth_projector::...` to `logos_core::net_worth_projector::...`.
