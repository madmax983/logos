# 🗣️ Echo: Jargon in CLI output

**🤦 The Confusion:**
When I run simple commands like `budget set` or `report month`, the CLI spits out messages like "Index restoration completed successfully", "Temporal index restored", and "Loaded temporal adjacency index from disk". I'm trying to set a budget, not travel through time! What is a temporal adjacency index?!

**🕵️ The Reality:**
The application is just loading the local database history, but it's using intense, jargon-heavy terminology ("temporal adjacency index") instead of simple words that normal people understand.

**💡 The Fix:**
Replace these confusing logs with something simple like "Loaded history" or "Database loaded", or just hide these startup messages entirely unless in debug mode.

**✅ Resolution:**
This issue was caused by the now-removed embedded database backend (`AletheiaDB`). Following the architectural migration to Postgres (`logos-store-pg`), these internal indices and their initialization jargon no longer exist in the codebase or the standard CLI output. The issue is resolved.
