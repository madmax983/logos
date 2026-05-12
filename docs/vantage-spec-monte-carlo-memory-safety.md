# 🔭 Vantage: Spec for Monte Carlo Memory Safety

👤 **User Story:**
"As a user running advanced probabilistic financial models, I want the Monte Carlo simulator to safely cap memory allocation limits, so that running extremely large simulations doesn't crash the application by exhausting system memory."

🤔 **So What? (Business Problem):**
Financial simulations are only as reliable as their execution environment. Currently, users can request an arbitrarily large number of simulation paths, which directly translates to massive memory allocations. When the system attempts to allocate more RAM than available, it triggers a catastrophic system crash. To be an enterprise-grade tool, we must anticipate extreme inputs and bound them safely, preventing Out-Of-Memory (OOM) errors and preserving the user's data and session.

🎯 **Metric Definition:**
Success = 0 system crashes or OOM panics when a user requests an extremely large number of paths (e.g., `u32::MAX`). The system should safely clamp the request to a sensible maximum or return a clear error.

🔍 **Gap Analysis:**
The Monte Carlo engine currently trusts user input blindly and attempts to pre-allocate vector capacity `Vec::with_capacity()` directly based on the requested number of paths. There is no hard limit or safety check. We need a defensive programming approach that sets a logical maximum for concurrent paths to guarantee stability.

✅ **Acceptance Criteria:**
- The Monte Carlo simulator must not crash or trigger a memory allocation failure when passed an extremely large number of simulation paths.
- The system must enforce a safe maximum limit on the number of paths allocated in memory.
- The simulator must gracefully handle edge-case inputs without panicking.

🚫 **Out of Scope:**
- Implementing disk-backed, out-of-core processing for infinitely large simulations.
- Discussing specific data structures or implementation details.
