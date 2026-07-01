# 🗣️ Echo: Financial planning examples are broken

**🤦 The Confusion:**
I tried to copy-paste the `RsuAutoDistributor`, `FireSimulator`, and `NetWorthProjector` examples directly from `docs/financial-planning.md`. The compiler immediately failed with errors like `module 'domain' is private` and `module 'planning' is private`. I thought the library was fundamentally broken or I was using the wrong version.

**🕵️ The Reality:**
Turns out the code examples are using internal/private module paths (e.g., `logos_core::domain::account::AccountId` and `logos_core::planning::fire::FireSimulator`). Users are supposed to import them directly from the root re-exports instead (like `logos_core::AccountId` and `logos_core::fire::FireSimulator`).

**💡 The Fix:**
Update the code snippets in `docs/financial-planning.md` to use the correct public import paths so they actually compile when copy-pasted.
