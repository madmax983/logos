# 🗣️ Echo: Cannot view budgets after setting them

**🤦 The Confusion:**
I used `budget set` to set my budget for the month. Then I wanted to double-check what I set it to, so I tried `budget get --month 2026-03` and `budget show --month 2026-03`. The CLI just gave me `Error: unknown subcommand 'get' for command 'budget'`. How am I supposed to view my budgets?

**🕵️ The Reality:**
The `budget` command only has a `set` subcommand. There is no command to simply list or show configured budgets without running a full financial report.

**💡 The Fix:**
Add a `budget get`, `budget show`, or `budget list` command so users can actually read the budgets they've written.
