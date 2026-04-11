# 🔭 Vantage: Spec for FIRE Ascent Overflow Safety

👤 **User Story:**
"As a user charting my FIRE (Financial Independence, Retire Early) journey, I want the system to handle extreme variables without crashing, so that even highly speculative or edge-case projections generate a valid scenario or an understandable error, rather than killing the simulation abruptly."

🤔 **So What?**
Users often stress-test their assumptions with very large numbers (e.g., massive hypothetical expenses paired with ultra-conservative withdrawal rates) to find the absolute boundaries of their FIRE plan. When a simulator panics due to unexpected numeric boundaries, it destroys confidence in the tool. Graceful error handling or bounds-checking ensures users trust the software for robust financial planning.

🎯 **Metric Definition:**
Success = 0 application panics when running the FIRE Ascent Simulator with edge-case or massive expense values. The simulation must complete, returning an explicit bounds error, capping at a maximum value, or correctly failing the ascent cleanly without a 64-bit integer multiplication overflow panic.

🔎 **Gap Analysis:**
The current `FireAscentSimulator` is vulnerable to arithmetic overflow during the milestone calculation. Specifically, when computing `camp3 = (fire_number * 3) / 4`, it fails to guard against a `fire_number` large enough to exceed `i64::MAX` when multiplied by 3. Robust financial calculators must use safe operations (e.g., saturating multiplication or checking bounds before intermediate arithmetic steps) to ensure stability.

✅ **Acceptance Criteria:**
- The `FireAscentSimulator` must not panic when computing milestones (e.g., Camp 1, 2, 3), regardless of how large the generated `fire_number` is.
- Intermediate multiplication steps (such as `fire_number * 3`) must be protected using saturating arithmetic or order-of-operation adjustments to prevent overflow.
- The `fire_ascent_havoc` test simulating large inputs must pass without triggering a panic.

🚫 **Out of Scope:**
- Transitioning the core financial math library to an arbitrary-precision decimal type.
- Implementing UI changes or CLI warnings beyond standard error propagation if an ascent is deemed "impossible" due to size constraints.
