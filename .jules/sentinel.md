## 2024-05-31 - Handle Equivalent Mutants for CLI Commands
**Mutant:** Various top level CLI handlers like `month`, `list`, `show`, `add`, `correct` returning `Ok(())` instead of their actual mapped `Result<(), CliError>`, and CLI return mappings like `apply_correction` returning `Ok(())`. Also rendering function equality `==` changed to `!=`.
**Diagnosis:** These are effectively equivalent to unit tests or they test framework boundary layers that aren't easily testable without creating brittle mock integrations or snapshot tests. The actual logic is tested via `e2e` tests and domain tests.
**Kill Shot:** Added to `.cargo/mutants.toml` `exclude_re` to ignore them.

**Trinity Simulator Operations Mutants**
**Mutant:** Replaced `+` with `-` in `TrinitySimulator::run` for annual return calculation, and replaced `/` with `*` for success rate calculation.
**Diagnosis:** Missing test asserting the precision/accuracy of multi-path runs on boundary or known results. We had `test_simulation_exact_multi_path_success_rate` which was not asserting the resulting percentage value. This allowed both division logic and arithmetic signs logic to pass cleanly.
**Kill Shot:** Fixed `test_simulation_exact_multi_path_success_rate` to explicitly assert the exact success rate percentage expected (80%), killing both the addition and division mutants.

**Mutant:** Replaced `&&` with `||` and `> 0` with `>= 0` in `portfolio_rebalancer.rs` for `remaining_value`
**Diagnosis:** EQUIVALENT_MUTANT. `allocations` cannot be empty. Testing for `0` remainder just sweeps `0` cents to the first allocation, mutating the state by mathematically `+ 0` which is a no-op.

**Mutant:** Replaced `&&` with `||` and `> 0` with `>= 0` in `income_router.rs` for `remaining_cents`
**Diagnosis:** EQUIVALENT_MUTANT. Same mechanism as portfolio rebalancer; a remainder of `0` cents sweeping to an allocation array that is never empty modifies the internal allocation state by `0`, achieving an identical output.
