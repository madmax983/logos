# 🔭 Vantage: Spec for FIRE Ascent Simulator Overflow Safety

👤 **User Story:**
"As a user planning my financial independence, I want the FIRE ascent simulation to safely handle extreme inputs without crashing, so that edge-case scenarios do not break my planning tools."

🤔 **So What?**
A financial simulation tool must be resilient to all inputs. If a user explores an extreme scenario (e.g., massive monthly expenses combined with a conservative withdrawal rate), the resulting FIRE number calculations could overflow standard 64-bit integer limits. A hard crash destroys trust in the platform's stability. Gracefully handling large numbers by capping them ensures the software remains reliable even under stress.

🎯 **Metric Definition:**
Success = 0 panics during FIRE ascent simulation, even when the `fire_number` exceeds what can be safely multiplied within `i64` limits. The report must generate successfully with capped values or explicit bounds errors.

🔎 **Gap Analysis:**
The `FireAscentSimulator` currently relies on unguarded multiplication, such as `(fire_number * 3) / 4`, which panics in debug/test environments (and wraps/overflows in release) when the intermediate product exceeds the 64-bit integer limit. Robust systems use saturating operations or safe accumulators for financial calculations to prevent system crashes on edge cases.

✅ **Acceptance Criteria:**
- The `FireAscentSimulator`'s milestone calculations (like `camp3`, `camp2`, `camp1`) must not panic or cause an arithmetic overflow crash when dealing with large FIRE numbers.
- The calculation logic must use safe bounds (e.g., saturating math or explicit checked math) to prevent crashes.
- All existing tests, including `fire_ascent_havoc`, must pass successfully.

🚫 **Out of Scope:**
- Transitioning the simulation engine to use arbitrary precision data types.
- Implementing complex UI warnings for saturated values in the CLI/TUI.
