# 🗣️ Echo: "Haircut" terminology is confusing

**🤦 The Confusion:**
I saw the terms "haircut-adjusted net worth" and `set_haircut_tiers(HaircutTierTable)` in the FIRE Simulator documentation. I thought this was some feature to track my personal grooming expenses, but the example applied it to my RSU vests.

**🕵️ The Reality:**
It turns out "haircut" is financial jargon for a risk-based discount applied to an asset's value. The tool reduces the projected value of future RSUs to account for market volatility.

**💡 The Fix:**
Rename `HaircutTierTable` to something intuitive like `RiskDiscountTable` or `VolatilityDiscount`, and update the docs to say "risk-adjusted value" instead of "haircut-adjusted". I am a normal person, I don't work on Wall Street.
