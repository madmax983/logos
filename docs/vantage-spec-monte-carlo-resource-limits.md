# 🔭 Vantage: Spec for Monte Carlo Simulator Resource Limits

👤 **User Story:**
As a user simulating market outcomes, I want the system to safely restrict absurdly large input values, so that my device does not crash from out-of-memory errors.

🤔 **So What?**
Financial forecasting tools must be robust and protect the user's computer from resource exhaustion. Allowing a user to request millions or billions of simulation paths can inadvertently consume all system memory, crashing the application. Providing safe constraints builds trust and ensures the tool remains stable even with extreme inputs.

📈 **Metric Definition:**
- Success = 0 memory exhaustion crashes when requesting an extremely high number of simulation paths.
- Performance = The simulator gracefully rejects input paths beyond a safe upper limit.

🔎 **Gap Analysis:**
The current simulation engine trusts the user input completely and attempts to process an unbounded number of simulation paths, leading to memory overflow crashes. A production-ready tool needs to enforce sensible limits to protect system stability.

✅ **Acceptance Criteria:**
- The simulator must reject requests for a number of paths that exceed a safe system maximum.
- The simulator must return a clear, user-friendly error message when limits are exceeded, rather than crashing.

🚫 **Out of Scope:**
- Optimizing the simulation logic itself for faster execution.
- Implementing disk-backed storage for extremely large simulation sets.
