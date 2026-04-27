🗣️ Echo: Getting Started example is broken

🤦 **The Confusion:**
Tried to run the `README.md` Getting Started block. The `docker compose up -d db` command ran successfully, but the following `cargo run -p logos-cli -- db migrate` command immediately crashed with a Postgres connection refused error, making it look like the tool was broken.

🕵️ **The Reality:**
Postgres inside the docker container takes a few seconds to actually start up and accept connections. The previous `sleep 3` command in the `README.md` was not long enough.

💡 **The Fix:**
Add a huge banner in README saying 'REQUIRES FEATURE NOVA' or tell users to wait a few seconds before running the migration, or better yet, add a `sleep 10` between the docker command and the migrate command in the example block so it works reliably when copy-pasted.
