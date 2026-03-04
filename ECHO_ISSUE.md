# 🗣️ Echo: FIRE Simulator example is broken

**🤦 The Confusion:**
Tried to run the `FireSimulator` example from `docs/financial-planning.md`. Compiler said `struct \`UpcomingVest\` has no field named \`gross_value_cents\``.

**🕵️ The Reality:**
Turns out the `UpcomingVest` struct expects `avg_close_price_cents` and `units` instead of a single `gross_value_cents` field. The documentation is showing an old or incorrect API.

**💡 The Fix:**
Update the `UpcomingVest` initialization in the `docs/financial-planning.md` example to use the correct fields. For example, replacing `gross_value_cents: 10000 * 100` with `avg_close_price_cents: 100 * 100` and `units: 100`.
