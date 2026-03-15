# Spec: Generic Aletheia Server Fallback Path

## 👤 User Story
As a first-time user exploring the CLI, I want the `aletheia start` command to have a sensible, system-agnostic default path for the `AletheiaDB` server manifest, so that I can easily launch the local server without encountering errors about developer-specific hardcoded paths.

## ✅ Acceptance Criteria
- The default path for launching the Aletheia server must be a relative, generic path (e.g., `../gallifreydb/Cargo.toml`) rather than an absolute path tied to a specific developer's machine.
- The path must be overridable via the `ALETHEIADB_MANIFEST_PATH` environment variable.
- Attempting to start the server must present a clear error message indicating the path it tried to use if the manifest is not found.
- The command must work identically across different operating systems without path formatting issues.

## 🚫 Out of Scope
- Automatically downloading or cloning the `AletheiaDB` repository if the manifest is missing.
- Interactive configuration prompts for setting the manifest path.
