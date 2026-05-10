**Refactoring `logos-store-pg`'s God Functions**
**Learning:** `logos-store-pg/src/store.rs` contains massive methods like `write_import_batch` and `write_reconciliation_run_and_month_close` which act as God Functions. They do complex validation, setup, payload generation, and multi-table transactions in one method. This results in pyramid of doom, huge function lengths (>130 lines), and passing references around heavily. The clippy warning `clippy::too-many-lines` keeps flagging them.
**Action:** Extract the payload construction or validation logic into separate helper functions to reduce the size of the transactional boundary functions. Flatten early exits and use idiomatic refactors.

**Refactoring God Functions in PostgresStore**
**Learning:** Extracting parts of "God Functions" (like `write_import_batch` and `write_reconciliation_run_and_month_close`) requires careful attention to where the extracted helper methods are placed. Placing helper methods inside trait implementations (`impl LedgerStore for PostgresStore`) causes compiler errors. Extracting purely data-mapping logic (like `fn map_statement_line`) avoids complicated lifetime/borrow-checker issues associated with attempting to extract both string allocations and the rows referencing them simultaneously.
**Action:** When breaking down massive functions, prefer extracting pure data mappers into `const fn` (where applicable) on the struct's main inherent `impl` block.
**Refactoring God Functions in CLI/TUI**
**Learning:** Functions like `fire_sim` and `render` were combining business logic/state assembly with complex UI rendering (creating and populating comfy_table models). This caused them to exceed 100 lines and become hard to read.
**Action:** Extract the rendering logic into helper functions (e.g. `render_fire_sim_output`, `render_runs_table`, `render_evidence_table`). Pass only the minimal necessary state into the rendering helpers.

**Idiomatic Closures for `needless_pass_by_value` in mapped iterators**
**Learning:** When resolving Clippy's `needless_pass_by_value` on functions that map over an iterator (especially ones optimized with `.into_iter()` to consume the collection), blindly changing the iterator to `.iter()` breaks the performance optimization. Using the `From` trait is the most idiomatic fix (`impl From<Row> for StoredObject`), but if you must pass a reference, use `.into_iter().map(|r| func(&r))` to keep the consumption while passing the reference.
**Action:** When updating function signatures from value to reference due to clippy, review the call sites. If mapping over an iterator, ensure you maintain the original `.into_iter()` (if it exists for optimization) by passing references inside the closure, or prefer implementing `From`/`Into`.

**Refactoring Simulation Loops and Math Functions**
**Learning:** Simulation methods like `run` in `trinity_simulator.rs` and `monte_carlo.rs` or `ascend` in `fire_ascent.rs` often combine initialization, tight loop iteration logic (or edge-case short circuits), and final metric calculation, leading to functions approaching 80+ lines.
**Action:** Extract the inner path generation loop into a `simulate_path` helper method. Extract milestone assembly or percentile logic into dedicated helper functions. This drastically cuts down `run` method sizes, flattening pyramid-of-doom structures without changing mathematical outcomes.
