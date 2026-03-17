# 🔭 Vantage: Spec for AletheiaDB Fallback Dependency

👤 **User Story:**
"As a developer evaluating `logos`, I want to be able to compile and run the CLI out of the box without manually cloning external databases, so that my getting started experience is seamless and does not immediately fail."

✅ **Acceptance Criteria:**
- **Dependency Resolution:** The `logos-store-aletheia` crate must not rely on a hardcoded local path (like `{ path = "/tmp/gallifreydb" }`) for the `aletheiadb` dependency.
- **Out of the box Compilation:** Ensure `aletheiadb` is pulled from a public source like a git repository or crates.io, preventing local path dependency errors for new developers.
- **Zero Configuration:** The application must compile and run `cargo build` without requiring any local paths or environment variables to be set by default.

🚫 **Out of Scope:**
- Removing `aletheiadb` completely (it is required for embedded storage).
