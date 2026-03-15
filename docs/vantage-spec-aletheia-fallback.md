# 🔭 Vantage: Spec for Optional AletheiaDB Dependency

👤 **User Story:**
"As a new Developer/User evaluating Logos, I want to be able to compile and run the CLI out-of-the-box without needing to manually clone or configure AletheiaDB, so that I can experience the core functionality immediately without compilation failures."

✅ **Acceptance Criteria:**
- **Success Metric:** `cargo run -p logos-cli` executes successfully on a fresh clone without `ALETHEIADB_MANIFEST_PATH` set or a local `aletheiadb` directory.
- The codebase defaults to pulling `aletheiadb` from a remote registry or git branch (instead of hardcoding a local path like `/tmp/gallifreydb` or `../gallifreydb/Cargo.toml`).
- If a user runs local server commands (e.g., `aletheia start`), it gracefully provides a clear, actionable error message if the local manifest cannot be resolved, rather than hard crashing with internal OS errors.

🚫 **Out of Scope:**
- Publishing AletheiaDB to crates.io (if it is a private/external repo, we simply rely on a git URL).
- Rewriting the storage backend to use a completely different database.
