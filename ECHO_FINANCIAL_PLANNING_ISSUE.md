# 🗣️ Echo: Financial Planning code examples fail to compile

🤦 **The Confusion:**
I was trying out the new financial planning features by copy-pasting the example code straight from `docs/financial-planning.md` into my `main.rs` file. When I tried to compile it, rustc threw a bunch of `error[E0603]: module 'planning' is private` and `module 'domain' is private` errors. The examples told me to use `logos_core::planning::fire::FireSimulator`, `logos_core::planning::net_worth_projector::NetWorthProjector`, `logos_core::domain::account::AccountId`, etc., but I just couldn't access them because the modules are private.

🕵️ **The Reality:**
The `planning` and `domain` modules in `logos_core` are actually defined as `pub(crate)`. Their contents or submodules are publicly re-exported at the crate root. For instance, `planning`'s submodules are re-exported via `pub use planning::*;` in `lib.rs`, which means `fire` and `net_worth_projector` are available at `logos_core::fire` and `logos_core::net_worth_projector`. Types like `AccountId` and `AllocationPolicy` are directly re-exported at the root level (`logos_core::AccountId`). The documentation still uses the direct, private module paths. Users shouldn't have to read the library source code to figure out the real public API paths.

💡 **The Fix:**
Update the code examples in `docs/financial-planning.md` to use the correct public re-exports. Change the imports from:
- `logos_core::planning::fire::{FireSimulator, UpcomingVest}` to `logos_core::fire::{FireSimulator, UpcomingVest}`
- `logos_core::planning::net_worth_projector::NetWorthProjector` to `logos_core::net_worth_projector::NetWorthProjector`
- `logos_core::domain::account::AccountId` to `logos_core::AccountId`
- `logos_core::domain::rsu::AllocationPolicy` to `logos_core::AllocationPolicy`
