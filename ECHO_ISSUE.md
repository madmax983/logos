# 🗣️ Echo: Getting Started example is broken (Resolved)

**🤦 The Confusion:**
I wanted to try out the `aletheia start` server feature mentioned in the `README.md`. So I copied the command: `cargo run -p logos-cli -- aletheia start`.
Instead of starting the server, cargo threw an error saying `failed to start aletheia server: manifest not found at 'C:\Users\markm\gallifreydb\Cargo.toml'`.
Why is the CLI looking for a manifest path inside a random user's `C:\` drive? I don't even use Windows!

**🕵️ The Reality:**
The `aletheia start` command and its hardcoded Windows path have already been removed entirely from the codebase in a recent storage layer cutover to Postgres+Diesel. The issue no longer applies to the current main branch as the `logos-store-aletheia` backend was deleted.

**💡 The Fix:**
None required. The issue is considered resolved by the recent architectural changes removing the old prototype storage engine.
