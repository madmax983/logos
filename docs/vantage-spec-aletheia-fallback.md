# 🔭 Vantage: Spec for AletheiaDB Fallback

👤 **User Story:**
"As a first-time user evaluating Logos, I want to be able to run the basic example commands without my compilation or CLI failing due to a missing local `gallifreydb` folder, so that I can experience the tool's value immediately."

✅ **Acceptance Criteria:**
- **Compilation:** `cargo run` commands in the README must compile and succeed out-of-the-box for someone who strictly cloned `logos` without downloading an external `AletheiaDB` repository.
- **Embedded Mode Default:** The `logos-store-aletheia` dependency must use a valid remote Git reference (e.g. `https://github.com/.../AletheiaDB.git`) rather than a hardcoded local path, so that cargo handles compilation automatically.
- **CLI Startup Resilience:** The `aletheia start` command must emit a human-readable instructional error message rather than referencing a hardcoded developer path (like `C:\Users\markm\gallifreydb\Cargo.toml`) when `ALETHEIADB_MANIFEST_PATH` is unassigned.
- **Success Metric:** A fresh clone with no local `AletheiaDB` source must be able to compile the CLI and execute `cargo run -p logos-cli -- txn add ...` without failing.

🚫 **Out of Scope:**
- Automating the `git clone` of the AletheiaDB repository.
- Modifying AletheiaDB itself.
