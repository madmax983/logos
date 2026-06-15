# 🗣️ Echo: Financial Planning examples do not compile

**🤦 The Confusion:**
I copied the code examples from `docs/financial-planning.md` into my project to try out the `FireSimulator`, `NetWorthProjector`, and `RsuAutoDistributor`. When I ran `cargo check`, the compiler gave me several errors like `module \`planning\` is private` and `module \`domain\` is private`. The examples wouldn't even compile!

**🕵️ The Reality:**
The `planning` and `domain` modules are declared as `pub(crate)` internally in `logos_core/src/lib.rs`. However, their types are re-exported at the crate root. The examples in the documentation incorrectly try to import from the private module paths (e.g., `use logos_core::planning::fire::FireSimulator;` and `use logos_core::domain::account::AccountId;`) instead of the public root paths.

**💡 The Fix:**
Update all code examples in `docs/financial-planning.md` to use the correct root-level imports without the `planning::` and `domain::` prefixes (e.g., `use logos_core::fire::FireSimulator;` and `use logos_core::AccountId;`).
