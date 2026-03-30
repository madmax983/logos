# 🔭 Vantage: Spec for Generic AletheiaDB Manifest Path Fallback

👤 **User Story:**
"As a new developer or user of Logos, I want the `aletheia start` command to either work out of the box or fail with a clear instruction, so that I don't waste time debugging errors caused by hardcoded, machine-specific paths belonging to other developers."

💼 **Business Problem:**
The `aletheia start` command currently fails out of the box for anyone except the original developer ("markm") because it defaults to an absolute path on their `C:\` drive (`C:\Users\markm\gallifreydb\Cargo.toml`). This immediately breaks the "Getting Started" experience, erodes trust in the tool's reliability, and creates unnecessary friction during onboarding. Complexity and friction are costs. We need the tool to be universally usable without requiring immediate configuration overrides.

✅ **Acceptance Criteria:**
- **Success Metric:** Running `cargo run -p logos-cli -- aletheia start` without setting `ALETHEIADB_MANIFEST_PATH` does not reference developer-specific absolute paths (e.g., `C:\Users\markm\...`).
- The default fallback for `ALETHEIADB_MANIFEST_PATH` must be a generic relative path (e.g., `../gallifreydb/Cargo.toml`) that assumes a standard adjacent-directory workspace layout.
- If the manifest is not found at the resolved path, the error message must remain clear and instruct the user to override the path using the `ALETHEIADB_MANIFEST_PATH` environment variable.

🚫 **Out of Scope:**
- Automatically cloning or downloading the `AletheiaDB` repository if it is missing.
- Changing how the `aletheia start` command invokes `cargo run`.
- Supporting environments where standard relative paths cannot be resolved.
