## 2026-04-12 - [Fire Ascent Simulator - Edge Case Coverage]
**Learning:** `FireAscentSimulator::ascend` has an uncovered branch handling the case where `fire_number == i64::MAX`. This happens when `safe_withdrawal_rate_pct` is set to `0`, making FIRE impossible. Adding a test for this proves the system correctly identifies and handles impossible ascents.
**Action:** When dealing with simulators that calculate targets based on rates, explicitly test the boundary conditions like `0%` rates to ensure the system gracefully handles impossible states.

## 2026-04-12 - [Excluding Equivalent Mutants]
**Learning:** Some mathematical boundary constraints (`amount > 0` becoming `amount >= 0`) create unviable or equivalent mutants because passing `0` to constructors (like `Posting::debit`) inherently causes domain errors that are cleanly caught and propagated via `?` up the stack.
**Action:** Added regex exclusions to `.cargo/mutants.toml` to safely ignore these known unviable permutations.
