# 🗣️ Echo: Financial Planning examples do not compile

**🤦 The Confusion:**
I tried doing the "README Run" by copy-pasting the `RsuAutoDistributor` example from `docs/financial-planning.md` directly into a fresh `src/main.rs`. When I tried to compile it, my terminal immediately threw multiple errors like `error[E0603]: module 'domain' is private` and `error[E0603]: module 'planning' is private`. I felt stupid and thought I had downloaded the wrong version of the library or broken my cargo setup, because the examples in the official docs wouldn't even compile.

**🕵️ The Reality:**
I did the "Import Scan" and discovered that the `logos-core` crate hides the `domain` and `planning` modules internally using `pub(crate)`. The structs and methods are re-exported at the crate root or submodules, but the documentation examples are explicitly importing them via their internal private paths (e.g., `use logos_core::planning::rsu_distributor::RsuAutoDistributor;` and `use logos_core::domain::account::AccountId;`).

**💡 The Fix:**
Update the code examples in `docs/financial-planning.md` to use the correct public export paths. For instance, change `use logos_core::domain::account::AccountId;` to `use logos_core::AccountId;`, `use logos_core::domain::rsu::AllocationPolicy;` to `use logos_core::AllocationPolicy;`, and `use logos_core::planning::rsu_distributor::...` to `use logos_core::rsu_distributor::...`.
