# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
As a user running Monte Carlo investment simulations, I want the simulator to gracefully handle extreme path count requests without crashing, so that my system remains stable even if I accidentally enter a massive simulation size.

🤔 **So What?**
Financial simulation tools must be resilient against out-of-bounds parameters. When users request an abnormally large number of simulation paths, failing to validate the input causes the system to attempt a massive memory allocation, resulting in a severe system crash (Out Of Memory panic). Crashing undermines user trust and reliability.

📈 **Metric Definition:**
- Success = 0 system crashes or out-of-memory panics when requesting massive path counts in Monte Carlo simulations. The system gracefully returns an error instead of terminating.

🔎 **Gap Analysis:**
The current projection engine attempts to reserve memory based on a fully user-provided path count without validation. Since this input can be extremely large, it translates to an immediate request for an unrealistic amount of system memory (e.g., dozens of gigabytes), causing a fatal system crash. We lack sensible input validation or bounded execution for simulations.

✅ **Acceptance Criteria:**
- The simulation engine must not crash or panic when provided with massive path values.
- The system must enforce a safe upper bound on the number of simulated paths (e.g., maximum of 1,000,000 paths).
- If the requested paths exceed the safe upper bound, the simulation must gracefully return a clear, user-facing error indicating the limit.

🚫 **Out of Scope:**
- Transitioning the simulation engine to stream results to disk to support infinitely large path counts.
- Rewriting the allocation strategy for reasonable path counts.
