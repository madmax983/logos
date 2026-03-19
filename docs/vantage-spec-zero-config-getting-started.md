# 🔭 Vantage: Spec for Zero-Config Getting Started

👤 **User Story:**
"As a new user evaluating Logos, I want to be able to clone the repository and run basic CLI commands immediately without complex manual configuration, so that I can experience the value of the tool without wrestling with hardcoded paths, external local repository requirements, or confusing technical jargon."

✅ **Acceptance Criteria:**
- **Success Metric:** A fresh clone of the repository must successfully compile and execute `cargo run -p logos-cli -- help` and `cargo run -p logos-cli -- txn add ...` without requiring modifications to `Cargo.toml` or creating external directories (e.g., `/tmp/gallifreydb`).
- **Dependency Resolution:** If the `aletheiadb` storage engine is required, it must be fetched via standard package management (e.g., crates.io or git URL) or included in the workspace, rather than relying on an absolute local path (`/tmp/...`).
- **Path Resolution:** Default configuration and manifest paths (such as the default for `ALETHEIADB_MANIFEST_PATH`) must use sensible, OS-agnostic relative paths or default user directories (e.g., `~/.logos/` or `%USERPROFILE%\.logos\`) instead of hardcoded developer paths (e.g., `C:\Users\markm\...`).
- **Jargon Removal:** CLI startup and execution output must use clear, human-readable terminology (e.g., "Loaded history" instead of "Loaded temporal adjacency index from disk"). Technical logs should be hidden behind a debug or verbose flag.

🚫 **Out of Scope:**
- Building a full graphical installer or standalone binary release process.
- Replacing the `aletheiadb` storage engine entirely; the focus is solely on making its inclusion and initialization frictionless.
- Writing extensive new end-user tutorials beyond fixing the immediate onboarding friction.