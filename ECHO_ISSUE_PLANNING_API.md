# 🗣️ Echo: Financial Planning API examples do not compile

🤦 **The Confusion:**
"I tried to run the examples from `docs/financial-planning.md` and `run_echo_test.sh` exactly as written, but the compiler threw `error[E0603]: module 'planning' is private` and `module 'domain' is private`. I have to guess how to import things like `AccountId` or `FireSimulator`."

🕵️ **The Reality:**
"The `domain` and `planning` modules are marked as `pub(crate)` in `logos-core`, so the paths in the documentation (`logos_core::domain::...` and `logos_core::planning::...`) are invalid for external users. The items are actually re-exported at the root level (e.g., `logos_core::fire::FireSimulator`), but the docs don't reflect this."

💡 **The Fix:**
"Update the examples in `docs/financial-planning.md` and `run_echo_test.sh` to use the correct public re-exports (e.g., `use logos_core::AccountId;` and `use logos_core::fire::FireSimulator;`) instead of the private module paths."
