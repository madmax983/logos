1. Remove the redundant `&& months < 1200` from `runway_simulator.rs` because it is covered by the `if current_burn_cents == 0 { return }` check.
2. Remove the `&& !allocations.is_empty()` check from `income_router.rs`.
3. Add `test_zero_amount_allocation_is_skipped` in `income_router.rs` to catch the `amount > 0` mutant.
4. Update exact assertion values for `test_snowball_vs_avalanche` in `debt_optimizer.rs` to assert `total_months` and `total_interest_paid_cents`.
5. Add `test_export_net_worth_timeline_negative` in `mermaid_xy_exporter.rs` to catch the `y_min` / `y_max` mutant.
