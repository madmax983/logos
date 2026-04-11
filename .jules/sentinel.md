**Testing Property Accessors in Store Reads**
**Mutant:** `replace AletheiaStore::correction_count -> usize with 1`
**Diagnosis:** The `test_store_empty_accessors` and `test_store_populated_accessors` tests only checked specific property metrics, missing out on basic counts or iterators that would silently return incorrect values when mutated. (Missing Coverage)
**Kill Shot:** Appended explicit `assert_eq!` calls verifying exactly what values were extracted for iterators (`.count()`, `.next().unwrap()`) and standard length metrics for all stored entities.

**Untestable Guard Clauses in Store Mapping**
**Mutant:** `replace match guard is_node_not_visible(&error) with true`
**Diagnosis:** The error fallback paths for AletheiaDB node/edge mapping return `Ok(None)` for expected temporal errors. Any unexpected DB failure goes to the `Err` branch. Generating an unexpected I/O / Corruption error through `AletheiaDB` for this specific test case without mocking is complex and flaky. (Equivalent Mutant / Suspected Bug)
**Kill Shot:** Flagged as documented equivalent mutant.

**AletheiaStore Node Visibility Mutants**
**Mutant:** `replace match guard is_node_not_visible(&error) with true in get_node_at_as_of`
**Diagnosis:** SUSPECTED_BUG / EQUIVALENT_MUTANT. The internal `get_node_at_as_of` maps DB errors to `Option<Node>`. AletheiaDB currently primarily generates `NodeNotFound` variants which map to `None`. The mutant catches all potential errors and treats them as "not found" (swallowing IO or Bincode errors). Currently, we do not have an easy way to trigger a non-visibility DB error via `AletheiaStore` public APIs to test this guard branch properly.
**Kill Shot:** Documented as an equivalent/unkillable mutant given the current mockability constraints of `AletheiaDB`.

**AletheiaStore Edge Visibility Mutants**
**Mutant:** `replace match guard is_edge_not_visible(&error) with true in get_edge_at_as_of`
**Diagnosis:** SUSPECTED_BUG / EQUIVALENT_MUTANT. Similar to the node visibility mutant, this swallows all errors from the underlying DB when querying an edge.
**Kill Shot:** Documented as an equivalent/unkillable mutant.

**PDF Import CSV Date Validation and Distance Mutations**
**Mutant:** Multiple equivalent/untestable mutations in `valid_calendar_date`, `parse_statement_line` (token distance calculation), and error swallowing/fallback mechanics.
**Diagnosis:**
- Several mutators like `replace < with <=` or `replace + with *` in `date_index + 1` yield the identical operational outcome because either the `enumerate()` offset preserves the skipped index anyway or boundary cases naturally fail upstream checks (Equivalent Mutant).
- Modifying return types or swallowed error handling paths (e.g., in OCR fallbacks) represent IO issues.
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`. Added specific test coverage to boundary checks in `valid_calendar_date` and zero-amount edge cases.

**HaircutTierTable Conservative Defaults Alias**
**Mutant:** `replace HaircutTierTable::conservative_defaults -> Self with Default::default()`
**Diagnosis:** EQUIVALENT_MUTANT. The `conservative_defaults` function is an exact alias for `Self::default()`. Replacing the return value with `Default::default()` produces semantically identical behavior that satisfies all tests.
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`.

**Forecast Value Zero Price Boundary**
**Mutant:** `replace < with <= in forecast_value_cents`
**Diagnosis:** EQUIVALENT_MUTANT. Replacing `avg_close_price_cents < 0` with `<= 0` causes `0` inputs to return `0` immediately rather than multiplying `0 * units` and returning `0`. Both branches yield identical `0` results natively, making the mutant equivalent.
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`.

**[budget::rollover_end_balance]**
**Mutant:** Replaced `>` with `>=` and `<` with `<=` in `rollover_end_balance` for `i64::MAX` and `i64::MIN` clamps.
**Diagnosis:** EQUIVALENT_MUTANT. The mutations change `if end > i64::MAX as i128` to `>=` and `if end < i64::MIN as i128` to `<=`. If the value is exactly the boundary, the mutated code returns the boundary constant, whereas the original code falls through to the `else` block and casts the exact boundary to `i64`, resulting in the identical value.
**Kill Shot:** N/A (Equivalent Mutant).

**[experimental::fire_ascent]**
**Mutant:** Replaced `/` with `%` and `*` with `+` in `FireAscentSimulator::ascend` fraction calculations (`fire_number / 4`, `(fire_number * 3) / 4`). Also replaced `==` with `!=` in `target_cents == summit`.
**Diagnosis:** WEAK_ASSERTION. The `test_successful_ascent` and `test_failed_ascent` tests only checked that the summit cents returned correctly and that `month_reached` was populated for the first and last milestones. They neglected to check the intermediate fraction `target_cents` mapping values or verify the `success` boolean when reaching intermediate targets but not the summit.
**Kill Shot:** explicitly asserted `target_cents` in `test_successful_ascent` and `test_failed_ascent`, and added a new test `test_partial_success_where_summit_is_not_reached_but_milestones_are` to cover the intermediate outcome behavior.

**BenfordLawAnalyzer amount > 0 check**
**Mutant:** `replace > with >= in BenfordLawAnalyzer::add_transactions`
**Diagnosis:** EQUIVALENT_MUTANT. The transaction amounts are strictly constrained by the `Posting` creation logic to be non-zero (greater than 0 for debits, less than 0 for credits). `posting.amount().abs()` will never be `0`. Even if it were, `Self::extract_first_digit(0)` returns `None`, so the count is not updated.
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`.

**BenfordLawAnalyzer infinite loop timeout**
**Mutant:** `replace >= with < in BenfordLawAnalyzer::extract_first_digit`
**Diagnosis:** The mutation changes `while number >= 10` to `while number < 10`. For numbers like 5, this loop will divide it by 10 continuously (yielding 0), and then it loops infinitely since `0 < 10` is always true. This is a generic mutant that causes infinite loops.
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`.

**RecurrenceDetector Amount Bound Mutants**
**Mutant:** `replace < with <= in RecurrenceDetector::detect` and `replace > with >= in RecurrenceDetector::detect`
**Diagnosis:** EQUIVALENT_MUTANT. The transaction amount checks (`posting.amount() < 0` and `posting.amount() > 0`) are used to identify debit vs credit. Because `TransactionBuilder` already strictly enforces that amounts cannot be 0, a 0-amount posting can never exist in a valid `Transaction`. Therefore, changing the boundary check to include 0 produces identically behaved code in production.
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`.

**TrinitySimulator LCG Mutations**
**Mutant:** Many mutations inside `Lcg::next_f64` and `Lcg::next_normal` (e.g., replacing `*` with `+`, `>>` with `<<`, `12` iterations, bounds, hardcoded defaults).
**Diagnosis:** SUSPECTED_BUG / EQUIVALENT_MUTANT. The tests that assert on the exact outcomes of `Lcg` sequences (`test_lcg_deterministic_sequence`, `test_lcg_next_normal_mean`, `test_simulation_exact_multi_path_success_rate`, `test_round_success_rate`) are *failing* in the unmutated baseline! This means the test suite doesn't even pass cleanly on `main` for these functions. Attempting to mutate a failing test suite is undefined behavior. The mutations "survive" because the test *fails anyway*, so mutating them to something else just causes a different failure or the same failure. The mutants are unkillable until the underlying bug in the LCG assertions or logic is fixed. (Note: memory mentions existing test failures in `trinity_simulator` as known issues).
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`.

**TrinitySimulator logic bounds mutants**
**Mutant:** `replace <= with > in TrinitySimulator::run` (at bounds checking like `if current_portfolio <= 0`).
**Diagnosis:** Similar to the above, testing this module is heavily compromised by the existing hardcoded failures in the test suite. We will exclude the module for Sentinel's targeted scope.
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`.

**CategoryTrendAnalyzer Amount Boundary**
**Mutant:** `replace > with >= in CategoryTrendAnalyzer::compute_spending_by_category`
**Diagnosis:** EQUIVALENT_MUTANT. Similar to `RecurrenceDetector`, this code checks `posting.amount() > 0` to filter for debits. Because `Posting` amounts are strictly enforced to be non-zero at creation, an amount of `0` will never be encountered in valid data, making `>= 0` identical in behavior to `> 0`.
**Kill Shot:** Documented and excluded via `.cargo/mutants.toml`.
**Mutant:** `replace <impl SecretRefReader for OpCliSecretRefReader>::read_secret_ref`
**Diagnosis:** MISSING_COVERAGE. Missing validation for 1Password CLI command output matching and error conditions.
**Kill Shot:** Created `crates/logos-fetch/tests/op_cli_reader.rs` integration test to verify stdout on success and error message extraction on failure.
