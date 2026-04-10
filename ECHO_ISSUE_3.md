# 🗣️ Echo: Jargon in CLI analytics arguments

**🤦 The Confusion:**
When trying to use the `analytics` subcommand, specifically `analytics snapshot create`, I am presented with arguments like `--as-of-valid-us <i64>` and `--as-of-tx-us <i64>`. I don't know what "valid-us" or "tx-us" means! Are these timezone offsets? Time limits?

**🕵️ The Reality:**
These refer to bitemporal database terminology: "Valid Time" and "Transaction Time", expressed in microseconds (`us`). This is intense, jargon-heavy terminology that normal people don't understand, especially when they just want to take an analytics snapshot.

**💡 The Fix:**
Simplify these arguments to be more human-readable, for example `--effective-time-us` and `--recorded-time-us`, or even better, just allow `--effective-time <ISO8601>` and parse it automatically, rather than requiring microseconds from the user.
