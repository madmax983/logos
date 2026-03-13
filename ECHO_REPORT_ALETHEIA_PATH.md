# 🗣️ Echo: aletheia start example fails due to placeholder path

**🤦 The Confusion:**
In the `README.md`'s "Run Local AletheiaDB" section, the example commands instruct the user to run `export ALETHEIADB_MANIFEST_PATH="/path/to/gallifreydb/Cargo.toml"`, followed by `cargo run -p logos-cli -- aletheia start`. Copying and pasting this exact block crashes with `failed to start aletheia server: manifest not found at '/path/to/gallifreydb/Cargo.toml'`.

**🕵️ The Reality:**
The `/path/to/gallifreydb/Cargo.toml` path provided in the documentation is a literal placeholder that does not exist on a user's machine, causing a frustrating failure on their first try running the server example.

**💡 The Fix:**
Update the `README.md` to either omit the `export` line (falling back to a default relative path if one exists) or explicitly instruct the user to replace the placeholder with the absolute path to their actual `AletheiaDB` checkout before running.
