# 🗣️ Echo: `aletheia start` example is broken out of the box

**🤦 The Confusion:**
I followed the "README Run" exactly and copy-pasted `cargo run -p logos-cli -- aletheia start` into my terminal. Instead of starting a local database server, it crashed with:
`Error: failed to start aletheia server: manifest not found at '../AletheiaDB/Cargo.toml' (override with ALETHEIADB_MANIFEST_PATH)`

Then I tried `cargo run -p logos-cli -- aletheia status` to see if it was already running, and got:
`Error: failed status check at 'http://127.0.0.1:8080/status': connection failed: Connection refused (os error 111)`

Why do I have to set an environment variable or have a specific folder structure `../AletheiaDB/` just to run the getting started example?

**🕵️ The Reality:**
The `README.md` examples do not work out of the box because the CLI assumes the external `AletheiaDB` repository is checked out in a specific relative path `../AletheiaDB/Cargo.toml`. If it's not there, it errors out, expecting the user to manually set `ALETHEIADB_MANIFEST_PATH`.

**💡 The Fix:**
Add a huge banner or clear prerequisite step in the `README.md` immediately before the `cargo run -p logos-cli -- aletheia start` command stating that the user must clone `AletheiaDB` first, or provide a default configuration that falls back to an embedded version automatically without requiring external repository paths.