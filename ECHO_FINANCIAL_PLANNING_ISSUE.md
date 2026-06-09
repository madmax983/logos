# 🗣️ Echo: Financial Planning docs examples don't compile and use heavy jargon

**🤦 The Confusion:**
I tried to follow `docs/financial-planning.md` to set up an RSU auto distributor and FIRE simulation. I copied the exact code from the examples, but it wouldn't even compile! The compiler kept yelling `error[E0603]: module 'planning' is private` when trying to import `logos_core::planning::...`. Also, the docs keep talking about "haircuts" (`HaircutTierTable`, "haircut-adjusted net worth"). I'm trying to plan my finances, why am I at a barber shop?

**🕵️ The Reality:**
The `planning` module in `logos_core` is kept as `pub mod` to avoid `clippy::redundant_pub_crate` warnings, but its contents are re-exported at the root. The examples in the docs still try to import through the private `planning` path instead of using the direct root exports (like `logos_core::FireSimulator`). In addition, "haircut" is heavy financial jargon for a risk-adjusted discount on an asset's value, which is confusing for normal users.

**💡 The Fix:**
- Update all the `use` statements in the examples in `docs/financial-planning.md` to point to the correct, accessible root paths (e.g., `use logos_core::{FireSimulator, NetWorthProjector, RsuAutoDistributor}`).
- Replace the "haircut" jargon with something plain English, like "discount", "risk-adjusted value", or "conservative estimate", so normal users can understand it easily.
