🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:**
Tried to run the code snippets from `docs/financial-planning.md`. The compiler said `error[E0603]: module 'planning' is private` when trying to run `use logos_core::planning::fire::{FireSimulator, UpcomingVest};` and other imports.

🕵️ **The Reality:**
Turns out I needed to import them directly from `logos_core` (e.g., `use logos_core::fire::{FireSimulator, UpcomingVest};`) because `planning` is declared as `pub(crate) mod planning;` inside `logos-core` but its contents are re-exported.

💡 **The Fix:**
Update the docs in `docs/financial-planning.md` to show the correct `logos_core::` imports instead of the private `logos_core::planning::` module.
