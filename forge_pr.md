⚒️ Forge: Refactor long functions in logos-core and logos-import

🚮 Smell: Functions like `ascend`, `run` in `monte_carlo.rs` and `trinity_simulator.rs`, and `parse_csv_columns` in `csv.rs` were > 60 lines and contained nested loops/matches (God Functions and Pyramid of Doom).
✨ Solution:
- `csv.rs`: Extracted inner match arms of `parse_csv_columns` into `handle_unquoted_char`, `handle_quoted_char`, and `handle_after_quote_char`.
- `fire_ascent.rs`: Flattened `ascend` by extracting `check_early_exits` and `build_milestones`.
- `monte_carlo.rs`: Extracted inner loop of `run` into `simulate_path`.
- `trinity_simulator.rs`: Extracted inner loop of `run` into `simulate_path`.
🧼 Benefit: Reduced cyclomatic complexity, improved readability, flattened structure, and resolved large function warnings.
🛡️ Verification: `cargo test` passes. No logic changed.
