## 2026-08-15 - [The "Black Box": FIRE module missing module-level example]
**Confusion:** The `crates/logos-core/src/planning/fire.rs` module explains its purpose but lacks a module-level `## Examples` block. A user looking at this file would see the abstract but no clear, top-down code snippet showing how the module is meant to be used holistically.
**Clarification:** Added a comprehensive module-level `## Examples` section demonstrating how to combine `FireSimulator::new`, `add_assets_liabilities`, `add_upcoming_vest`, and `fire_progress_pct` to get an actionable result.
