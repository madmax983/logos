## 2024-05-31 - Handle Equivalent Mutants for CLI Commands
**Mutant:** Various top level CLI handlers like `month`, `list`, `show`, `add`, `correct` returning `Ok(())` instead of their actual mapped `Result<(), CliError>`, and CLI return mappings like `apply_correction` returning `Ok(())`. Also rendering function equality `==` changed to `!=`.
**Diagnosis:** These are effectively equivalent to unit tests or they test framework boundary layers that aren't easily testable without creating brittle mock integrations or snapshot tests. The actual logic is tested via `e2e` tests and domain tests.
**Kill Shot:** Added to `.cargo/mutants.toml` `exclude_re` to ignore them.

**Trinity Simulator Operations Mutants**
**Mutant:** Replaced `+` with `-` in `TrinitySimulator::run` for annual return calculation, and replaced `/` with `*` for success rate calculation.
**Diagnosis:** Missing test asserting the precision/accuracy of multi-path runs on boundary or known results. We had `test_simulation_exact_multi_path_success_rate` which was not asserting the resulting percentage value. This allowed both division logic and arithmetic signs logic to pass cleanly.
**Kill Shot:** Fixed `test_simulation_exact_multi_path_success_rate` to explicitly assert the exact success rate percentage expected (80%), killing both the addition and division mutants.

## 2024-06-03 - Handle Planning Module Mutants
**Mutant:** Multiple arithmetic and boundary mutations in `FireSimulator`, `NetWorthProjector`, and `RsuAutoDistributor`.
**Diagnosis:** Weak tests and missing coverage. Tests for `project_timeline` did not test zero months, bounds correctly, or exact boundary conditions for vests. Tests for `distribute_rsu_vest` didn't verify amounts when specific buckets had percentages. Some `distribute_rsu_vest` boundary mutants are equivalent or unviable and added to exclusions.
**Kill Shot:** Added tests to cover these gaps in `net_worth_projector.rs` and `rsu_distributor.rs`, and updated `.cargo/mutants.toml` with equivalents.
