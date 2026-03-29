# 🗣️ Echo: README Run for local AletheiaDB fails due to placeholder path

🤦 **The Confusion:**
I copied and pasted the example from the README to run the local AletheiaDB:
`export ALETHEIADB_MANIFEST_PATH="/path/to/gallifreydb/Cargo.toml"`
`cargo run -p logos-cli -- aletheia start`
It immediately crashed saying `manifest not found at '/path/to/gallifreydb/Cargo.toml'`. I don't have a folder named `path/to/gallifreydb`.

🕵️ **The Reality:**
The README uses a literal `/path/to/...` placeholder in a copy-pasteable code block. Users who just copy-paste without reading carefully will end up setting a bad environment variable that breaks the default fallback.

💡 **The Fix:**
Remove the `export` line from the code block, or put it in a separate block clearly marked as an example, and just let users rely on the default `../AletheiaDB/Cargo.toml` behavior out-of-the-box for the "happy path".
