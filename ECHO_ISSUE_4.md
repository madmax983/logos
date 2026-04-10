# 🗣️ Echo: Getting Started error messages are missing helpful action

**🤦 The Confusion:**
When running simple commands like `txn add` without starting the Postgres database, I get this error:
`Error: Command 'txn.add' failed at runtime: runtime initialization failed: failed to connect to postgres: connection to server at "127.0.0.1", port 5432 failed: Connection refused`

While it does ask `Is the server running on that host and accepting TCP/IP connections?`, it doesn't give me any specific, helpful steps to resolve it within the context of the application.

**🕵️ The Reality:**
The application relies heavily on Postgres being up and properly configured.

**💡 The Fix:**
Add a line to the error output specifically telling the user *how* to start the database using the provided `docker-compose` setup. For example: "If you are running locally, try `docker compose up -d db`".
