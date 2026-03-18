# 🔭 Vantage: Spec for Aletheia DB Fallback

👤 **User Story:**
"As a new developer or user onboarding to the `logos` project, I want to build and run the CLI out of the box without being forced to manually clone an external database repository (`aletheiadb`), so that I can immediately evaluate the core tool without encountering confusing path-not-found build errors."

✅ **Acceptance Criteria:**
- **Success Metric:** 100% of new clones of `logos` successfully compile and execute `cargo run -p logos-cli --help` without requiring the `ALETHEIADB_MANIFEST_PATH` environment variable or an external `/tmp/gallifreydb` folder to exist.
- The default configuration in `crates/logos-store-aletheia/Cargo.toml` must not hardcode an absolute local filesystem path (like `/tmp/gallifreydb` or `C:\Users\markm\...`).
- If `aletheiadb` is a mandatory dependency, the dependency should point to an accessible crates.io version, a git repository URL, or optionally be hidden behind a `cargo` feature flag (e.g., `feature = ["local-db"]`) that is disabled by default.
- If the user explicitly attempts to run the `aletheia start` server command without the local source available, the CLI must return a helpful, actionable error message instructing them *how* to set `ALETHEIADB_MANIFEST_PATH` rather than panicking with an `os error 2`.

🚫 **Out of Scope:**
- Rewriting the core storage engine to eliminate `aletheiadb` entirely.
- Publishing `aletheiadb` to crates.io if it is currently a private or experimental internal tool (a git dependency is an acceptable alternative).
- Automatically cloning the `aletheiadb` repository on behalf of the user during the build process.
