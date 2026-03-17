# 🔭 Vantage: Spec for Out-of-the-Box Aletheia Integration

👤 **User Story:**
"As a first-time Logos user, I want the project to compile and run out of the box without encountering hardcoded local path errors for `aletheiadb`, so that I can quickly evaluate the tool without debugging custom developer environment setups."

✅ **Acceptance Criteria:**
- **Compilation:** The CLI and codebase must successfully build and pass tests (`cargo build`, `cargo test`) on any machine without requiring the `gallifreydb` source code to be manually cloned to specific hardcoded directories like `/tmp/gallifreydb`.
- **Server Execution Fallback:** The `aletheia start` command default manifest path must be generic (e.g., `../gallifreydb/Cargo.toml`) rather than user-specific absolute paths (e.g., `C:\Users\markm\gallifreydb\Cargo.toml`).
- **Clear Documentation:** If `aletheiadb` is an optional feature or integration, the `README.md` must accurately reflect this and provide explicit, fail-proof instructions on how to configure the environment or install dependencies if the user opts in to running the local Aletheia server.
- **Fail Gracefully:** Commands relying on the local Aletheia server must output actionable, descriptive errors instead of confusing `os error 2` filesystem errors when the `ALETHEIADB_MANIFEST_PATH` is missing or invalid.

🚫 **Out of Scope:**
- Replacing AletheiaDB with a different storage backend (e.g., SQLite or PostgreSQL).
- Automatically downloading the `gallifreydb` repository from Git during the build process.
- Writing the engineering fix for the `logos-store-aletheia` dependency configuration (this spec only defines the required behavior).
