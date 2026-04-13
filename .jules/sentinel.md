## 2024-05-24 - Missing Coverage in CSV Parsing Edge Case

**Mutant:** Replaced `==` with `!=` in `parse_simple_csv_row` (`max_idx == usize::MAX`)
**Diagnosis:** MISSING_COVERAGE. The edge case where missing columns calculation evaluates `max_idx` to `usize::MAX` (representing bounds that can't be safely checked using regular column logic) wasn't explicitly tested.
**Kill Shot:** Added tests `test_csv_missing_columns_usize_max_when_found_greater` and `test_csv_missing_columns_usize_max_minus_one` in `crates/logos-import/tests/csv_mapping_mutant.rs` to explicitly force this bounds behavior and assert the correct `expected` output value from the error.

## 2024-05-24 - Equivalent Mutants in Budget Rollover

**Mutant:** Replaced `>` with `>=` and `<` with `<=` in `rollover_end_balance`.
**Diagnosis:** EQUIVALENT_MUTANT. Because the checks compare an `i128` against `i64::MAX as i128` and `i64::MIN as i128`, modifying them to `>=` or `<=` correctly clamps the value to `i64::MAX` or `i64::MIN` without altering the behavior (since `i64::MAX as i64` == `i64::MAX`).
**Kill Shot:** Added to `.cargo/mutants.toml` skip list.

## 2024-05-24 - Suspected Unviable Test due to External Dependency

**Mutant:** Migrations in `logos-store-pg` (`run_pending_migrations`, `pending_migration_names`).
**Diagnosis:** UNVIABLE in current sandbox. Modifying these correctly fails tests, but we can't write/run verifying tests here because `testcontainers` requires a live Docker Hub Postgres pull which hits rate limits.
**Kill Shot:** Added to `.cargo/mutants.toml` skip list.
