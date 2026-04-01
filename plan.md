1. **Create `transactions.rs`**
   - Extract `post_double_entry`, `transaction_exists`, `transactions_as_of_us`, `apply_correction`, `latest_correction_target` into a new `impl AppRuntime { ... }` block in `crates/logos-runtime/src/runtime/transactions.rs`.
   - Update `mod.rs` to include `pub mod transactions;`.

2. **Verify `transactions.rs` creation**
   - Run `ls crates/logos-runtime/src/runtime/` to confirm `transactions.rs` exists.
   - Run `cargo check -p logos-runtime` to ensure imports are satisfied and the split didn't break anything. Add any necessary `use` statements to `transactions.rs`.

3. **Create `budgets.rs`**
   - Extract `budget_variance_for`, `set_budget_target_for_month`, `budget_target_for_month`, `budget_variance_for_month`, `month_report_for`, `plan_rsu_budget_for_month`, `expense_total_for_month`, `transaction_in_month` into `budgets.rs`.
   - Update `mod.rs` to include `pub mod budgets;`.

4. **Verify `budgets.rs` creation**
   - Run `ls crates/logos-runtime/src/runtime/` to confirm `budgets.rs` exists.
   - Run `cargo check -p logos-runtime`. Add necessary `use` statements to `budgets.rs`.

5. **Create `reconcile.rs`**
   - Extract `reconcile_month_for`, `reconcile_and_persist_month_for`, `reconciliation_run_count`, `reconciliation_run`, `list_reconciliation_runs`, `close_month`, `month_close_for_scope`, `statement_lines_for_reconciliation_run`, `reconciliation_transaction_ids_for` into `reconcile.rs`.
   - Update `mod.rs` to include `pub mod reconcile;`.

6. **Verify `reconcile.rs` creation**
   - Run `ls crates/logos-runtime/src/runtime/` to confirm `reconcile.rs` exists.
   - Run `cargo check -p logos-runtime`. Add necessary `use` statements to `reconcile.rs`.

7. **Create `fetch.rs`**
   - Extract `fetch_run`, `list_fetch_runs`, `stage_fetched_statement_artifact`, `run_month_autopilot`, `fetch_configured_statement_artifacts`, `fetched_statement_artifact_for`, `import_fetched_statement_artifact`, `resolve_autopilot_balances`, `load_statement_source_config`, `secret_bundle_for_fetch_source`, `secret_bundle_for_fetch_source_with_resolver`, `run_fetch_adapter`, `persist_fetch_run_from_result`, `persist_failed_fetch_run`, `fetch_error_to_runtime`, `output_format_label`, `store_fetch_run_status` into `fetch.rs`.
   - Update `mod.rs` to include `pub mod fetch;`.

8. **Verify `fetch.rs` creation**
   - Run `ls crates/logos-runtime/src/runtime/` to confirm `fetch.rs` exists.
   - Run `cargo check -p logos-runtime`. Add necessary `use` statements to `fetch.rs`.

9. **Create `import.rs`**
   - Extract `import_csv_row`, `import_csv_statement`, `import_pdf_statement`, `imported_record_count`, `post_import_record`, `import_content_hash_key_legacy_v1`, `import_content_hash_keys`, `import_batch_key`, `parse_import_timestamp` into `import.rs`.
   - Update `mod.rs` to include `pub mod import;`.

10. **Verify `import.rs` creation**
    - Run `ls crates/logos-runtime/src/runtime/` to confirm `import.rs` exists.
    - Run `cargo check -p logos-runtime`. Add necessary `use` statements to `import.rs`.

11. **Create `analytics.rs`**
    - Extract `default_analytics_schema_version`, `create_analytics_snapshot`, `list_analytics_snapshots`, `get_analytics_snapshot`, `snapshot_rows`, `hash_rows`, `write_rows_to_parquet`, `build_double_entry` into `analytics.rs`.
    - Update `mod.rs` to include `pub mod analytics;`.

12. **Verify `analytics.rs` creation**
    - Run `ls crates/logos-runtime/src/runtime/` to confirm `analytics.rs` exists.
    - Run `cargo check -p logos-runtime`. Add necessary `use` statements to `analytics.rs`.

13. **Clean up `mod.rs`**
    - Only leave struct definitions, `new`, `open`, `new_in_memory`, `default_store_path`, `register_balance_for`, `current_month_key_local`, `current_month_key_utc`, `default_artifacts_root`, `default_fetch_config_path`, `current_time_us`, `month_key_from_wallclock_utc`, `civil_from_days`, `build_double_entry` (if shared) in `mod.rs`. Wait, `build_double_entry` is used across modules, so keep it in `mod.rs` or move it to a shared `util.rs`. We will leave it in `mod.rs` for now or move it to a `util.rs`. Let's create `crates/logos-runtime/src/runtime/util.rs` for `build_double_entry`, `parse_import_timestamp`, `transaction_in_month`, `month_key_from_wallclock_utc`, `civil_from_days`.
    - Note: The exact list of remaining items in `mod.rs` will be adjusted during the actual extraction based on what is shared.
    - Ensure `cargo test --workspace` passes cleanly.

14. **Log architecture change**
    - Add the following exact content to `.jules/atlas.md`:
```markdown
**[The Blob Runtime]**
**Tangle:** `AppRuntime` in `crates/logos-runtime/src/runtime/mod.rs` was an over 1700 lines long "Blob" anti-pattern, handling unrelated concerns like transactions, budgets, reconciling, fetching, importing, and analytics.
**Blueprint:** Extracted the massive `impl AppRuntime` block into cohesive, domain-specific modules (`transactions`, `budgets`, `reconcile`, `fetch`, `import`, `analytics`), leaving only core setup and orchestration logic in `mod.rs`.
```

15. **Verify architecture log**
    - Run `cat .jules/atlas.md` to confirm the log was updated correctly.

16. Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.

17. Submit the PR with the architecture changes.
