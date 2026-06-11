# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
Tried to run the README commands. The migration command crashed with a connection refused error.

**🕵️ The Reality:**
The `docker compose up -d db` command fails because of a Docker unauthenticated pull rate limit when it attempts to pull the `postgres:16` image. Since the image fails to pull, the database container never starts, and therefore Postgres is never ready to accept connections.

**💡 The Fix:**
Include instructions on how to handle Docker rate limits or use a local Postgres installation in the README.
