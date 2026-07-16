# 🔭 Vantage: Spec for Monte Carlo Memory Safety

👤 **User Story:**
As a financial planner running probabilistic simulations, I want the system to gracefully reject excessively large simulation requests, so that my application does not crash from memory exhaustion.

🤔 **So What?**
When users request an unreasonably large number of simulation paths, the system attempts to allocate more memory than is physically available, resulting in a hard crash. This degrades user trust and makes the tool feel fragile. A resilient system should anticipate extreme inputs and enforce safe operational boundaries rather than terminating unexpectedly.

🎯 **Metric Definition:**
Success = 0 system crashes or panics when users request an extreme number of simulation paths. The system must instead return a clear, user-facing error message explaining the boundary limit.

🔍 **Gap Analysis:**
The current simulation engine tries to fulfill any requested number of paths without validating if the resulting memory footprint is feasible. Competitor products and professional financial tools enforce upper limits on simulation counts to guarantee stability and responsive performance.

✅ **Acceptance Criteria:**
- The simulation engine must not crash or terminate when provided with a massive number of requested paths.
- The system must enforce a reasonable maximum limit on the number of simulation paths allowed per run.
- If the requested number of paths exceeds the safe limit, the system must gracefully reject the request and return a clear error stating the calculation exceeds safe memory boundaries.

🚫 **Out of Scope:**
- Transitioning to an out-of-core or disk-backed simulation model to support infinite paths.
- Altering the mathematical model or percentile calculation logic of the simulations.
