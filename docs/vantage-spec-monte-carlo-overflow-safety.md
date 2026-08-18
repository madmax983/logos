# 🔭 Vantage: Spec for Monte Carlo Simulator Resilience

👤 **User Story:**
As a user running probabilistic simulations on my investments, I want the system to safely cap or reject excessively large projection paths without crashing, so that my terminal session remains stable even if I make a typo or test extreme boundaries.

🤔 **So What?**
A financial tool that panics on unexpected inputs is unreliable and untrustworthy. When users explore edge cases (e.g., millions of simulation paths), the software should gracefully report an error or cap results, rather than suffering a hard crash due to memory allocation failure that destroys the entire user session.

📈 **Metric Definition:**
Success = 0 panics during Monte Carlo simulations, even when providing inputs like `4294967295` (max 32-bit integer) paths. The simulation must complete or explicitly return a bounding error.

🔎 **Gap Analysis:**
Currently, the Monte Carlo projector attempts to allocate memory dynamically based directly on the user input without an upper bound check. This assumes the user will always input a reasonable number of paths. Robust systems prevent malicious or accidental inputs from causing massive out-of-memory errors by validating the inputs before allocation.

✅ **Acceptance Criteria:**
- The Monte Carlo simulator must not crash or panic due to memory allocation failure when given extremely large path counts.
- The simulator must check if the requested paths exceed a safe operational limit and return a clear, user-facing error if exceeded.
- All existing tests, including tests that trigger this massive allocation, must pass successfully after the fix by receiving the expected bounds error.

🚫 **Out of Scope:**
- Rewriting the entire simulator to stream results to disk instead of using in-memory vectors.
