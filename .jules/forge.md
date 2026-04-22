**Refactoring `logos-store-pg`'s God Functions**
**Learning:** `logos-store-pg/src/store.rs` contains massive methods like `write_import_batch` and `write_reconciliation_run_and_month_close` which act as God Functions. They do complex validation, setup, payload generation, and multi-table transactions in one method. This results in pyramid of doom, huge function lengths (>130 lines), and passing references around heavily. The clippy warning `clippy::too-many-lines` keeps flagging them.
**Action:** Extract the payload construction or validation logic into separate helper functions to reduce the size of the transactional boundary functions. Flatten early exits and use idiomatic refactors.

**Refactoring God Functions in PostgresStore**
**Learning:** Extracting parts of "God Functions" (like `write_import_batch` and `write_reconciliation_run_and_month_close`) requires careful attention to where the extracted helper methods are placed. Placing helper methods inside trait implementations (`impl LedgerStore for PostgresStore`) causes compiler errors. Extracting purely data-mapping logic (like `fn map_statement_line`) avoids complicated lifetime/borrow-checker issues associated with attempting to extract both string allocations and the rows referencing them simultaneously.
**Action:** When breaking down massive functions, prefer extracting pure data mappers into `const fn` (where applicable) on the struct's main inherent `impl` block.
**Refactoring God Functions in CLI/TUI**
**Learning:** Functions like `fire_sim` and `render` were combining business logic/state assembly with complex UI rendering (creating and populating comfy_table models). This caused them to exceed 100 lines and become hard to read.
**Action:** Extract the rendering logic into helper functions (e.g. `render_fire_sim_output`, `render_runs_table`, `render_evidence_table`). Pass only the minimal necessary state into the rendering helpers.
