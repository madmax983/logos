# 🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:**
Tried to run the example code from `docs/financial-planning.md`. The compiler threw multiple `module is private` errors when trying to import from `logos_core::domain` and `logos_core::planning`.

🕵️ **The Reality:**
Turns out the internal `domain` and `planning` modules are marked as `pub(crate)` and are not publicly exported at that path. Users are actually supposed to import these types directly from the crate root re-exports (e.g. `use logos_core::AccountId;`, `use logos_core::fire::FireSimulator;`, `use logos_core::net_worth_projector::NetWorthProjector;`).

💡 **The Fix:**
Update the examples in `docs/financial-planning.md` and any doc-comments to use the correct public export paths instead of the internal, private module paths.
