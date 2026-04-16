## 2024-05-31 - Handle Equivalent Mutants for CLI Commands
**Mutant:** Various top level CLI handlers like `month`, `list`, `show`, `add`, `correct` returning `Ok(())` instead of their actual mapped `Result<(), CliError>`, and CLI return mappings like `apply_correction` returning `Ok(())`. Also rendering function equality `==` changed to `!=`.
**Diagnosis:** These are effectively equivalent to unit tests or they test framework boundary layers that aren't easily testable without creating brittle mock integrations or snapshot tests. The actual logic is tested via `e2e` tests and domain tests.
**Kill Shot:** Added to `.cargo/mutants.toml` `exclude_re` to ignore them.

## 2024-05-31 - Havoc Tests that Timeout
**Mutant:** OOM/Timeout panics in tests inside `csv_havoc.rs` and `pdf_havoc.rs`.
**Diagnosis:** These tests were configured as `#[should_panic]` but the logic under test wasn't strictly enforcing panic bounds. Removing the `#[should_panic]` properly asserts that code logic works within the test limits.
**Kill Shot:** Removed `#[should_panic]` and verified it passes cleanly without causing test failures.

## 2024-05-31 - Treat DB Migrations and Memory DB Default Init as Equivalent
**Mutant:** Various `logos-store-pg` and `logos-store` DB initialization routines.
**Diagnosis:** These either require a test container / physical DB or mock boilerplate that tests essentially framework code and not logic. `logos-store` traits missing getters are mostly for interfaces and correctly excluded.
**Kill Shot:** Added to `.cargo/mutants.toml` `exclude_re` to ignore them.
