# 🗣️ Echo: Financial Planning docs examples fail to compile

**🤦 The Confusion:**
I copied the code examples directly from `docs/financial-planning.md` to try out the new `FireSimulator` and `NetWorthProjector` features. When I ran `cargo run`, the compiler threw a bunch of errors complaining about private modules, like `error[E0603]: module 'planning' is private` and `error[E0603]: module 'domain' is private`.

**🕵️ The Reality:**
The examples in the documentation use explicit import paths that include internal private modules (e.g., `use logos_core::planning::fire::FireSimulator;` and `use logos_core::domain::account::AccountId;`). In `logos-core`, modules like `planning` and `domain` are declared as `pub(crate)`, making them inaccessible from outside the crate.

**💡 The Fix:**
The examples in `docs/financial-planning.md` should be updated to use the public re-exports at the root of `logos_core`. For example:
- Replace `use logos_core::planning::fire::...` with `use logos_core::fire::...`
- Replace `use logos_core::domain::account::AccountId;` with `use logos_core::AccountId;`
- Replace `use logos_core::domain::rsu::AllocationPolicy;` with `use logos_core::AllocationPolicy;`
- Replace `use logos_core::planning::rsu_distributor::...` with `use logos_core::rsu_distributor::...`
- Replace `use logos_core::planning::net_worth_projector::...` with `use logos_core::net_worth_projector::...`
