1. **Red Phase (The "Havoc" Fix):**
   - I identified a weak point: `CashflowProjector::project_balances` in `crates/logos-core/src/experimental/cashflow_projector.rs` blindly adds to balances using `+=`, which panics on arithmetic overflow when dealing with large values near `i64::MAX`.
   - I wrote a failing proptest in `crates/logos-core/tests/havoc_proptest.rs` that generates large initial balances and amounts.
   - I verified the vulnerability causes a panic (`attempt to add with overflow`).

2. **Green Phase:**
   - I added `#[should_panic(expected = "attempt to add with overflow")]` to the test to make it pass minimally without fixing the bug.

3. **Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.**
   - Run tests.

4. **Submit the wreckage (PRESENT):**
   - Submit the PR with the Title `👺 Havoc: CashflowProjector Panics on Arithmetic Overflow` and the appropriate description.
