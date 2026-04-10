# 🗣️ Echo: Jargon in CLI output (Resolved)

**🤦 The Confusion:**
When I run simple commands like `budget set` or `report month`, the CLI spits out messages like "Index restoration completed successfully", "Temporal index restored", and "Loaded temporal adjacency index from disk". I'm trying to set a budget, not travel through time! What is a temporal adjacency index?!

**🕵️ The Reality:**
These jargon-heavy logs were emitted by the prototype AletheiaDB storage engine. This engine was recently removed entirely in a major architectural cutover to Postgres+Diesel. The current runtime does not emit these logs on startup.

**💡 The Fix:**
None required. The underlying code emitting this database jargon was deleted when removing the `logos-store-aletheia` backend.
