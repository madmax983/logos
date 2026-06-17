# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
"Tried to run the code blocks in `docs/financial-planning.md`. The compiler said `module 'domain' is private` and `module 'planning' is private` when importing things like `logos_core::domain::account::AccountId` and `logos_core::planning::fire::FireSimulator`."

**🕵️ The Reality:**
"Turns out the library re-exports everything at the root! I am supposed to just use `logos_core::AccountId` and `logos_core::FireSimulator` instead of the deeply nested private modules."

**💡 The Fix:**
"Update the examples in `docs/financial-planning.md` to import directly from the root re-exports, e.g., `use logos_core::AccountId;` and `use logos_core::FireSimulator;`."
