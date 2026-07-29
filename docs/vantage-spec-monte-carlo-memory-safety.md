# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
"As a user running Monte Carlo simulations, I want the system to safely handle inputs for an extremely large number of simulation paths without crashing, so that accidentally typing a huge number doesn't break the entire application."

🤔 **So What? (Business Problem):**
A financial tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., massive number of simulation paths), the software should gracefully report an error or cap the maximum allowed paths, rather than suffering a hard crash due to an out-of-memory error.

📈 **Metric Definition:**
Success = 0 panics during Monte Carlo simulations, even when providing inputs like `u32::MAX` for the number of paths. The simulation must return a clear error indicating the requested number of paths exceeds system limits.

🔎 **Gap Analysis:**
Currently, the Monte Carlo simulator unconditionally attempts to allocate memory proportional to the requested number of paths without bounding the input. This assumes the user will only request a reasonable number of paths. Standard financial simulation engines enforce safe maximum limits on simulation scales to prevent resource exhaustion.

✅ **Acceptance Criteria:**
- The Monte Carlo simulator must not panic when asked to simulate an extremely large number of paths.
- The simulator must enforce a safe upper bound on the number of requested simulation paths to prevent unbounded dynamic memory allocation.
- All existing tests, including chaos/proptests that trigger this overflow, must pass successfully after the fix.

🚫 **Out of Scope:**
- Rewriting the entire simulator to operate in a completely streaming fashion with zero allocations (Phase 2).
