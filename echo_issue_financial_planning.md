# 🗣️ Echo: Financial planning primitive examples fail to compile

🤦 **The Confusion:**
I am a new user trying to use the Financial Planning Primitives. I copy-pasted the examples for `RsuAutoDistributor`, `FireSimulator`, and `NetWorthProjector` from `docs/financial-planning.md` into my `main.rs`. When I tried to compile, none of the examples worked! The compiler threw multiple errors saying `module 'domain' is private` and `module 'planning' is private` for imports like `logos_core::planning::fire::FireSimulator` and `logos_core::domain::account::AccountId`.

🕵️ **The Reality:**
The examples in the documentation are using internal private module paths instead of the actual public API paths. The Rust compiler error itself points out that the modules are private and suggests importing from the root (e.g., `logos_core::rsu_distributor` instead of `logos_core::planning::rsu_distributor`).

💡 **The Fix:**
Update all the import statements in the `docs/financial-planning.md` examples to use the correct, publicly accessible module paths (e.g., `logos_core::fire::FireSimulator` and `logos_core::AccountId`) so that users can actually copy, paste, and run them without getting compilation errors.
