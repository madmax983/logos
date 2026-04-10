# 🔭 Vantage: Spec for Aletheia Server Path Configuration

👤 **User Story:**
"As a developer exploring the Logos CLI, I want the `aletheia start` command to work out-of-the-box using relative or standard paths, so that I don't have to manually override developer-specific hardcoded paths from the original author."

🤔 **So What?**
A CLI that fails to run basic tutorial or getting-started commands due to hardcoded machine-specific paths completely destroys the first-time user experience. Users will immediately abandon a tool that crashes out-of-the-box looking for a `C:\Users\markm` path when they are on Mac or Linux. Fixing this removes a massive friction point and makes the software actually usable for anyone other than the original developer.

🎯 **Metric Definition:**
Success = The `cargo run -p logos-cli -- aletheia start` command successfully boots up (or cleanly fails looking for a standard relative manifest) without referencing `C:\Users\markm\` anywhere in the error or execution path when run on a fresh clone.

🔎 **Gap Analysis:**
The application currently defaults `DEFAULT_ALETHEIA_MANIFEST_PATH` to a hardcoded Windows path (`C:\Users\markm\gallifreydb\Cargo.toml`). Standard CLI applications use relative paths (e.g., `../gallifreydb/Cargo.toml`) or resolve relative to the current working directory to ensure cross-platform compatibility.

✅ **Acceptance Criteria:**
- The default Aletheia manifest path must be changed from the hardcoded absolute Windows path to a generic relative path (e.g., `../gallifreydb/Cargo.toml`).
- The `aletheia start` command must not look for `C:\Users\markm\` on a fresh installation.
- Overriding the path via the `ALETHEIADB_MANIFEST_PATH` environment variable must still work as expected.

🚫 **Out of Scope:**
- Creating an automatic setup script for Aletheia DB if it doesn't exist locally.
- Modifying Aletheia server behavior beyond fixing the manifest path.
