1. **Add module documentation and doc-tests to `crates/logos-reporting/src/rsu_forecast.rs`:**
   - Add a module-level doc (`//!`) explaining its purpose (summarizing projected RSU events).
   - Add `///` docs to `RsuForecastSummary`, `project_rsu_forecast_summary`, and methods. Include a `## Examples` block with executable code for `project_rsu_forecast_summary`.
2. **Review other files in `logos-reporting`:**
   - They look fairly well documented based on previous grepping, but I'll quickly check `rsu_budget_plan.rs` for missing `//!` module doc.
3. **Check `logos-fetch` adapters:**
   - `crates/logos-fetch/src/adapters/provident.rs` and `crates/logos-fetch/src/adapters/mod.rs` already have `//!` comments.
4. **Complete pre-commit steps:**
   - Run `cargo fmt --all`.
   - Run `cargo clippy --workspace --all-targets --offline -- -D warnings`.
   - Run `cargo test --workspace --offline`.
   - Run `cargo doc --no-deps --workspace`.
5. **Submit a PR with Title "🎻 Bard: [documentation update]"**
