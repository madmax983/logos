# 🗣️ Echo: `cargo run -p logos-cli -- aletheia start` looks for a hardcoded Windows path

**🤦 The Confusion:**
Tried to run `cargo run -p logos-cli -- aletheia start` as suggested in the `README.md` (before exporting the `ALETHEIADB_MANIFEST_PATH`), and the command immediately crashed with:
`failed to start aletheia server: manifest not found at 'C:\Users\markm\gallifreydb\Cargo.toml' (override with ALETHEIADB_MANIFEST_PATH)`

**🕵️ The Reality:**
The default fallback path is hardcoded to a specific user's local Windows directory (`C:\Users\markm\gallifreydb\Cargo.toml`). This is highly specific and guarantees failure for any other developer or CI environment unless they manually set the environment variable.

**💡 The Fix:**
Either change the fallback to be a relative path (e.g., `../../../gallifreydb/Cargo.toml` as mentioned in the project memory), or just error cleanly without leaking a local dev machine's path like "Please set the ALETHEIADB_MANIFEST_PATH environment variable".
