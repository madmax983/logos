## 2026-04-12 - [Fire Ascent Simulator - Edge Case Coverage]
**Learning:** `FireAscentSimulator::ascend` has an uncovered branch handling the case where `fire_number == i64::MAX`. This happens when `safe_withdrawal_rate_pct` is set to `0`, making FIRE impossible. Adding a test for this proves the system correctly identifies and handles impossible ascents.
**Action:** When dealing with simulators that calculate targets based on rates, explicitly test the boundary conditions like `0%` rates to ensure the system gracefully handles impossible states.
## 2024-05-24 - Pre-existing Failing Tests Note
**Insight:** There are pre-existing tests like `parse_csv_columns_panics_on_oom` in `logos-import/tests/csv_havoc.rs` that fail on the main branch before any changes are made.
**Action:** Proceed with submission as these are known pre-existing test failures not caused by my changes.
