## 🤖 Sentinel: Killed mutants with deterministic output tests

### 🧬 Mutants Found
- Found 12 missed mutants in `crates/logos-cli/src/commands/analytics.rs`
- Found 7 missed mutants in `crates/logos-cli/src/commands/budget.rs`
- Found 1 missed mutant in `crates/logos-cli/src/commands/report.rs`
- Found 1 missed mutant in `crates/logos-cli/src/commands/reconcile.rs`
- Found 6 missed mutants in `crates/logos-store-pg/src/migrate.rs`
- Found 37 missed mutants in `crates/logos-store/src/memory.rs`

### 🎯 Tests Added/Strengthened
- Strengthened `render_month_output_is_deterministic`, `render_show_output_is_deterministic`, `render_budget_set_output_is_deterministic_zero_variance`, `render_budget_set_output_is_deterministic_positive_variance`, `render_snapshot_manifest_list_is_deterministic`, and `render_snapshot_manifest_is_deterministic` to assert the presence of expected header strings because mutants that deleted table headers (`table.set_header(...)`) were surviving.
- Added `render_fire_sim_output_is_deterministic` to cover `render_fire_sim_output` because none of the metrics/table values being rendered were actually tested.

### ⚠️ Suspected Bugs
None.

### 📊 Kill Rate
- The mutations tests are failing on `logos-tui` and `logos-store-pg` due to 60s timeout limits, so these aren't currently tested.
- `logos-cli` mutations tests dropped from 12 to 6 uncaught for `analytics.rs`.
- `logos-cli` mutations tests dropped from 7 to 6 uncaught for `budget.rs`
- `crates/logos-cli/src/commands/report.rs` mutants fully killed.
- `crates/logos-cli/src/commands/reconcile.rs` mutants fully killed.

### 🔗 Havoc Interaction
- None of the surviving mutants appear to cross paths with Havoc's tests (mostly deterministic table rendering logic or `Ok(())` wrapper returns on CLI runners).

