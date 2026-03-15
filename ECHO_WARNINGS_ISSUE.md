# 🗣️ Echo: Commands spit out scary warnings and debug logs

## 🤦 The Confusion
I ran the `txn add` command from the README. It added the transaction, but it printed this scary warning:
`Warning: Failed to load manifest: Missing required index file: manifest.idx`
Did I break the database? Am I missing a file? Is my data safe?!

Then I ran `month autopilot`, and it spammed my terminal with database internals:
`Index restoration completed successfully: 4 nodes, 2 edges loaded`
`Temporal index restored: 4 node versions, 2 edge versions`
`Loaded temporal adjacency index from disk`
I don't care about temporal graphs, nodes, or edges. I just want to run my finances! Why is the CLI showing me all these internals?

## 🕵️ The Reality
The embedded database `aletheiadb` logs debug info to stdout by default, and warns when a new DB is created (because `manifest.idx` doesn't exist yet). Users are seeing internal implementation details that look like errors.

## 💡 The Fix
Silence these debug logs in the CLI. The 'Warning: Failed to load manifest' should be completely hidden if it just means 'creating a new database'. Hide index restoration messages unless a `--verbose` flag is passed. Simple is better than powerful!
