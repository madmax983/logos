# 🗣️ Echo: README Getting Started commands fail on first run

**🤦 The Confusion:**
I wanted to try the tool by copying the commands directly from the `README.md`. I ran `docker compose up -d db` and immediately ran the next line, `cargo run -p logos-cli -- db migrate`. It instantly crashed with a huge red error: `Connection refused. Is the server running on that host and accepting TCP/IP connections?`. I thought the tool was completely broken or the docker setup was wrong.

**🕵️ The Reality:**
The `docker compose up -d db` command exits successfully as soon as the container is *created*, but Postgres takes a few seconds to actually start up and accept connections inside the container. Because I copy-pasted the whole block or ran the commands back-to-back, the CLI tried to connect before Postgres was ready.

**💡 The Fix:**
Add a small note in the README telling users to wait a few seconds before running the migration, or better yet, add a `sleep 3` between the docker command and the migrate command in the example block so it works reliably when copy-pasted.
