# 🔭 Vantage: Spec for Monte Carlo Simulator Resilience

👤 **User Story:**
As a user running financial simulations, I want the Monte Carlo simulator to gracefully reject excessively large numbers of simulation paths, so that the application does not completely crash or run out of memory.

🤔 **So What?**
Currently, users can request an astronomical number of simulation paths which attempts to allocate more memory than the system possesses. This results in an immediate crash. By imposing reasonable safety limits, we prevent catastrophic failures and ensure a stable, predictable user experience when running heavy simulations.

📈 **Metric Definition:**
Success = 0% crash rate due to memory allocation failures when running simulations with extreme inputs.

🔎 **Gap Analysis:**
The current simulation engine implicitly trusts user input for the number of paths, leading to unbounded memory requests. Competing consumer financial tools enforce strict maximums on simulation runs to preserve backend stability, whereas our local tool currently allows inputs that overwhelm the host machine.

✅ **Acceptance Criteria:**
- The simulator must define a sensible upper bound for the number of requested simulation paths.
- Requests exceeding this maximum must be rejected gracefully with a descriptive error message instead of crashing.
- Normal use cases (e.g., standard path counts) must continue to function without any degraded performance.

🚫 **Out of Scope:**
- Streaming simulation results to disk to allow infinite paths (we will simply cap memory usage in Phase 1).
- Changing the underlying mathematical model of the Monte Carlo simulation.
