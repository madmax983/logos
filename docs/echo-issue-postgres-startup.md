# 🗣️ Echo: Getting Started example fails with Connection Refused

🤦 **The Confusion**
I copy-pasted the exact Example Commands from the README to spin up the local database and run migrations, but it immediately failed with: `Error: Connection Refused: Postgres may still be starting up.`

🕵️ **The Reality**
The `sleep 3` command in the README isn't long enough for the Postgres container to finish initializing on my machine before the `db migrate` command fires.

💡 **The Fix**
Increase the sleep duration or use a proper healthcheck script so it actually waits for the database to be ready before running the migration command.
