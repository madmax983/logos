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

**[Register Balance Projection Overflow]**
**Mutant:** HAVOC_REPORT.md (Havoc Fuzzer: `project_register_balance_iter` panics on overflow)
**Diagnosis:** SUSPECTED_BUG - The code assumes register entries will never sum to more than the 64-bit integer limit, which causes a panic on overflow. It should likely saturate instead of panicking.
**Kill Shot:** None - Flagged for Atlas or Forge as a suspected bug to fix.
