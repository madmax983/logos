# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
"As a user running probabilistic financial projections, I want the system to safely reject impractically large simulation parameters, so that a typo or extreme input doesn't crash my machine or the application by exhausting all available memory."

🤔 **So What?**
A simulation tool that blindly allocates memory based on user input without constraints is a severe vulnerability and degrades trust. If a user accidentally requests `u32::MAX` paths, the system currently attempts to allocate gigabytes of RAM upfront and panics. We must define sensible upper bounds for our inputs or return explicit errors when limits are exceeded to ensure system stability and a professional user experience. Resilient software builds trust; fragile software creates frustration.

🎯 **Metric Definition:**
Success = 0 system panics or out-of-memory aborts during the Monte Carlo simulation, even when providing extreme inputs like `u32::MAX` paths. The simulation must return an explicit error instead of crashing.

🔎 **Gap Analysis:**
Currently, `MonteCarloProjector::run` attempts to pre-allocate memory for all requested paths using an unguarded capacity request (`Vec::with_capacity`). This assumes the number of paths requested will always fit comfortably within system RAM. Standard robust applications apply sensible bounds-checking and limits to user-provided parameters that directly control memory allocation to prevent resource exhaustion.

✅ **Acceptance Criteria:**
- The Monte Carlo Simulator must not panic or abort due to memory exhaustion when provided with extremely large path counts (e.g., `u32::MAX`).
- The system must define a safe upper limit for the number of paths and return a clear, explicit error if the user requests a number exceeding this limit.
- All existing tests, including chaos/proptests that trigger this memory overflow, must pass successfully after the fix (i.e., verifying the error is returned instead of a panic).

🚫 **Out of Scope:**
- Streaming the paths to disk to support arbitrarily large simulations.
- Discussing implementation details like specific Rust types or struct modifications used to fix the vulnerability.
