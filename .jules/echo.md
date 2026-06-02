**The Confusion:**
Documentation users copying examples directly from `docs/financial-planning.md` received "module is private" and "unresolved import" errors for `domain` and `planning` modules.

**The Reality:**
The code structure uses `pub(crate) mod domain;` and `pub(crate) mod planning;` inside `logos-core`, and then uses `pub use domain::...` and `pub use planning::*` at the crate root to re-export the items publicly. The documentation examples were directly targeting the internal module paths rather than using the public crate root exports.

**The Fix:**
Updated the examples in `docs/financial-planning.md` to use the correct `logos_core::` crate-level re-exports (e.g. `use logos_core::AccountId;`, `use logos_core::fire::FireSimulator;`) so that they accurately reflect the public API and compile when copy-pasted.
