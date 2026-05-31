# 🗣️ Echo: Financial Planning docs getting started example is broken

🤦 **The Confusion:**
Tried to run the example code from `docs/financial-planning.md`. The compiler said `module planning is private` and that it could not compile.

🕵️ **The Reality:**
Turns out `logos-core::planning` is private (`pub(crate)`) and its internals are re-exported at the crate root. The documentation uses the internal private module path `use logos_core::planning::fire::*` instead of the public re-exports `use logos_core::fire::*`.

💡 **The Fix:**
Update the documentation examples in `docs/financial-planning.md` to use the correct public re-export paths (e.g. `use logos_core::fire::*` and `use logos_core::net_worth_projector::*`).
