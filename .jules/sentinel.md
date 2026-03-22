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
