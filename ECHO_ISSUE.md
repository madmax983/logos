# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
I wanted to try out the `aletheia start` server feature mentioned in the `README.md`. So I copied the command: `cargo run -p logos-cli -- aletheia start`.
Instead of starting the server, cargo threw an error saying `failed to start aletheia server: manifest not found at 'C:\Users\markm\gallifreydb\Cargo.toml'`.
Why is the CLI looking for a manifest path inside a random user's `C:\` drive? I don't even use Windows!

**🕵️ The Reality:**
Turns out `crates/logos-cli/src/commands/aletheia.rs` had a hardcoded `DEFAULT_ALETHEIA_MANIFEST_PATH` pointing to `C:\Users\markm\gallifreydb\Cargo.toml`. Since I didn't explicitly set the `ALETHEIADB_MANIFEST_PATH` environment variable, it fell back to this developer-specific path and crashed my run.

**💡 The Fix:**
Change the default path to be a generic relative path, like `../gallifreydb/Cargo.toml`. This way it's a reasonable fallback for anyone trying to run the tool, instead of a confusingly specific path.
