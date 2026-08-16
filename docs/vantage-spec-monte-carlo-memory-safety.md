# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
"As an investor running a Monte Carlo simulation, I want the system to safely handle requests for an extremely large number of simulation paths without crashing, so that a typo or an intentional stress-test does not cause a hard memory failure that crashes the application."

🤔 **So What?**
Financial simulation tools must be resilient and reliable. If a user inputs an excessively large value for the number of paths (e.g., simulating millions or billions of paths), the system currently attempts to allocate an amount of memory that exceeds the physical RAM available. This results in a hard OS-level crash (memory allocation failure/OOM kill), which destroys the user's session and severely degrades trust in the tool's stability. Instead of crashing, the system must recognize unreasonable requests and handle them gracefully.

📈 **Metric Definition:**
Success = 0 panics during a Monte Carlo simulation, even when the requested number of paths exceeds maximum memory capacity. The simulation must either run within a constrained limit or return a clear error indicating the request is too large.

🔎 **Gap Analysis:**
Currently, the Monte Carlo simulation logic attempts to allocate memory proportional to the number of paths requested. When an excessively large number of paths is passed, it issues an unbounded allocation request, triggering a memory allocation panic. Standard financial planning software implements hard limits or iterative processing to prevent out-of-memory errors from large user inputs.

✅ **Acceptance Criteria:**
- The Monte Carlo simulator must not crash due to memory allocation failures when given an extremely large number of paths.
- The simulator must implement a safe upper bound on the number of paths it will attempt to allocate at once.
- If the requested paths exceed the safe upper bound, the simulator must return a clear, user-facing error message explaining the limit.
- All existing tests, including chaos/proptests simulating this failure, must pass successfully after the fix is implemented.

🚫 **Out of Scope:**
- Re-architecting the Monte Carlo simulator to run out-of-core (e.g., streaming results to disk instead of using RAM).
- Modifying the underlying statistical algorithms used in the simulation.
