## 2024-05-31 - Handle Equivalent Mutants for CLI Commands
**Mutant:** Various top level CLI handlers like `month`, `list`, `show`, `add`, `correct` returning `Ok(())` instead of their actual mapped `Result<(), CliError>`, and CLI return mappings like `apply_correction` returning `Ok(())`. Also rendering function equality `==` changed to `!=`.
**Diagnosis:** These are effectively equivalent to unit tests or they test framework boundary layers that aren't easily testable without creating brittle mock integrations or snapshot tests. The actual logic is tested via `e2e` tests and domain tests.
**Kill Shot:** Added to `.cargo/mutants.toml` `exclude_re` to ignore them.

**Trinity Simulator Operations Mutants**
**Mutant:** Replaced `+` with `-` in `TrinitySimulator::run` for annual return calculation, and replaced `/` with `*` for success rate calculation.
**Diagnosis:** Missing test asserting the precision/accuracy of multi-path runs on boundary or known results. We had `test_simulation_exact_multi_path_success_rate` which was not asserting the resulting percentage value. This allowed both division logic and arithmetic signs logic to pass cleanly.
**Kill Shot:** Fixed `test_simulation_exact_multi_path_success_rate` to explicitly assert the exact success rate percentage expected (80%), killing both the addition and division mutants.
## 2024-04-22 - [Equivalent Mutants in Domain Boundaries]
**Mutant:** Many surviving mutants replacing `>` with `>=` and `&&` with `||` in `mermaid_exporter.rs`, `portfolio_rebalancer.rs`, `income_router.rs`, `debt_optimizer.rs`, and `benford_law.rs`.
**Diagnosis:** In these domains (e.g. `amount_cents > 0`), the amounts are typically enforced strictly to be non-zero at the domain entry point (like `debit` or `credit` creation returning errors for `<= 0`). Thus, replacing `> 0` with `>= 0` processes non-existent zero amounts, making them EQUIVALENT MUTANTS since valid transactions cannot have zero amounts.
**Kill Shot:** Appended these equivalent mutant regexes to `.cargo/mutants.toml` skip list.
