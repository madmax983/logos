# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
I wanted to try out the `aletheia start` server feature mentioned in the `README.md`. So I copied the command: `cargo run -p logos-cli -- aletheia start`.
Instead of starting the server, cargo threw an error saying `failed to start aletheia server: manifest not found at '../AletheiaDB/Cargo.toml'`.
Why is the CLI looking for a manifest path in a completely different directory that doesn't exist?

**🕵️ The Reality:**
Turns out `crates/logos-cli/src/commands/aletheia.rs` has a hardcoded `DEFAULT_ALETHEIA_MANIFEST_PATH` pointing to `../AletheiaDB/Cargo.toml`. If I didn't clone the `AletheiaDB` repository right next to this one, it crashes my run unless I explicitly set the `ALETHEIADB_MANIFEST_PATH` environment variable.

**💡 The Fix:**
Change the default path to be a more robust fallback, or update the `README.md` to make it explicitly clear that `AletheiaDB` must be cloned as a sibling directory, rather than just showing the environment variable as "Optional".
