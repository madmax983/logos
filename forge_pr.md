⚒️ Forge: Refactor args.rs into args module

🚮 Smell: `logos-cli/src/args.rs` was a massive ~1300 line God Module mixing CLI models, routing logic, and string parsing.
✨ Solution: Split `args.rs` into `args/model.rs` and `args/parser.rs` and re-exported items through `args/mod.rs` to maintain public APIs.
🧼 Benefit: Significantly flattens the file and separates models from logic, dropping cognitive complexity and line counts.
🛡️ Verification: Cargo clippy strict lints and all workspace tests passed with no logic changes.
