# 🗣️ Echo: `aletheia start` fails out of the box because it expects `gallifreydb` checked out

**🤦 The Confusion:**
I followed the instructions in `README.md` to run the local AletheiaDB: `cargo run -p logos-cli -- aletheia start`.
Instead of starting the server, cargo threw an error: `failed to start aletheia server: manifest not found at '../gallifreydb/Cargo.toml' (override with ALETHEIADB_MANIFEST_PATH)`.
Why does a command to run a database server expect me to have cloned a completely different codebase (`gallifreydb`) right next to my current directory?

**🕵️ The Reality:**
Turns out `crates/logos-cli/src/commands/aletheia.rs` defaults to a sibling directory `../gallifreydb/Cargo.toml`. This assumes developers have cloned a specific other repository, breaking the "out-of-the-box" experience for anyone just trying to run the command from the README.

**💡 The Fix:**
Since `logos-store-aletheia/Cargo.toml` pulls `aletheiadb` from git (`https://github.com/madmax983/AletheiaDB.git`), the CLI shouldn't assume a local checkout exists. Either the `aletheia start` command should automatically clone/download it, or the `README.md` must clearly warn the user to clone `gallifreydb` to `../gallifreydb` *before* running the command.
