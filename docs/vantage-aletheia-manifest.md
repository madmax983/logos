# 🔭 Vantage: Spec for Generic Aletheia Manifest Path

## 👤 User Story
As a New Developer, I want the default AletheiaDB manifest path to use a generic, relative location rather than a developer-specific absolute path, so that I can seamlessly evaluate the `aletheia start` feature without encountering confusing, hardcoded file-not-found errors.

## ✅ Acceptance Criteria
- **Generic Pathing:** The default path for `ALETHEIADB_MANIFEST_PATH` must be `../gallifreydb/Cargo.toml` instead of an absolute Windows path.
- **Graceful Failure:** If the manifest is not found at the generic path, the CLI must output a clear error message that instructs the user on how to override it using the `ALETHEIADB_MANIFEST_PATH` environment variable.
- **Cross-Platform:** The fallback path should work transparently across different operating systems (Linux, macOS, Windows).

## 🚫 Out of Scope
- Automatic cloning of the `gallifreydb` repository if the path is missing.
- Advanced dynamic path resolution (e.g., scanning the filesystem for the correct `Cargo.toml`).
- Modifying how `AletheiaDB` itself manages its internal storage.
