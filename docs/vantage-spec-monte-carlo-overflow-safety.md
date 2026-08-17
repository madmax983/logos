# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Limits

👤 **User Story:**
As a user running probabilistic simulations, I want the system to safely cap the scale of simulation paths, so that requesting a massive projection doesn't crash the application by exhausting system memory.

🤔 **So What?**
Financial simulation tools must be resilient to all boundaries. Hard system crashes from memory exhaustion degrade user trust and makes the tool feel unreliable. When users input massive simulation path figures, encountering a panic destroys the entire user session. Providing bounded limits is critical for a professional-grade financial planner.

📈 **Metric Definition:**
Success = 0 system crashes or unexpected terminations during probabilistic simulation projection.

🔎 **Gap Analysis:**
The current engine dynamically calculates simulation paths without safely limiting the scale, exceeding capacity for massive inputs (e.g. `u32::MAX`). A robust engine needs to anticipate these edge cases and handle numeric expansion safely without hitting internal hard limits that lead to system panics.

✅ **Acceptance Criteria:**
- The engine must not crash when provided with massive simulation scales.
- The system must correctly intercept memory/capacity limits and return a clear, user-facing error stating the calculation exceeds safe boundaries.

🚫 **Out of Scope:**
- Transitioning to streaming computation.
- Altering fundamental projection algorithms for normal operational bounds.