🛡️ Sentry: [test coverage improvement]

🎯 Target: `logos-core` (multiple modules including `domain::correction`, `experimental::cashflow_projector`, `experimental::fire_ascent`, `experimental::mermaid_exporter`, `planning::fire`, and `planning::rsu_distributor`)
💣 Risk: Prevent logic bugs or panics from missed branches such as division-by-zero on empty inputs (`mermaid_exporter`), incorrect loops/skips (`cashflow_projector`), unhandled bounds (`fire_ascent` with `i64::MAX`), missed constructor errors (`rsu_distributor` empty description), and uncovered getters.
🧪 Strategy: Added specific unit tests to trigger the unexercised failure modes (invalid accounts, unbalanced transactions) to verify they fail closed/continue cleanly instead of panicking.
🔬 Verification: `cargo test -p logos-core`
