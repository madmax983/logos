🛡️ Sentry: [test coverage improvement]

🎯 Target: `logos-reporting` module boundary conditions, specifically `project_register_balance_iter`. `logos-core` experimental tests feature configurations, and `logos-store-pg` `migrate` coverage.
💣 Risk: Previously, `project_register_balance_iter` lacked test coverage verifying the mathematical bounds (i64 max/min saturation during register aggregation), representing an uncovered panic boundary in projection math. The experimental modules within `logos-core` were also being excluded from test configurations if the `nova` feature wasn't specifically provided, and the database migration functions `pending_migration_names` and `run_pending_migrations` were entirely missing execution coverage, allowing potential breaking changes in startup sequencing to go unnoticed.
🧪 Strategy:
- Added a targeted test `should_saturate_on_overflow_iter` within `crates/logos-reporting/src/register.rs` which explicitly iterates on `i64::MAX` and `i64::MIN` delta insertions to mathematically verify `saturating_add` correctness without panic.
- Modified `crates/logos-core/tests/portfolio_rebalancer_havoc.rs` and `crates/logos-core/tests/debt_optimizer_havoc.rs` to include `#![cfg(feature = "nova")]` enabling tests conditionally over experimental module dependency.
- Introduced `run_pending_migrations_works_and_pending_migration_names_works` within `crates/logos-store-pg/tests/migration_smoke.rs` using testcontainers to provide verifiable coverage against actual `postgres` instance startup flows.
🔬 Verification:
- `cargo test --workspace --all-targets --all-features`
