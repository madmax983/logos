# 🔭 Vantage: Spec for Monte Carlo Memory Safety

👤 **User Story:**
"As a user running probabilistic simulations, I want the system to safely handle requests for a massive number of paths without crashing, so that an accidental typo in the simulation parameters doesn't exhaust my system memory and crash the application."

🤔 **So What? (Business Problem):**
Robust financial tools must protect users from footguns. When a user requests an impossibly large number of simulation paths (like `u32::MAX`), attempting to allocate memory for all of them at once will crash the system due to Out Of Memory (OOM) errors. Gracefully rejecting unreasonably large requests or capping them ensures the software remains stable and trustworthy.

🎯 **Metric Definition:**
Success = 0 memory allocation panics during Monte Carlo simulation, even when providing massive inputs like `u32::MAX` for the number of paths. The simulation must complete or return an explicit bounds error.

🔎 **Gap Analysis:**
Currently, `MonteCarloProjector::run` attempts to allocate a vector with capacity equal to the requested number of paths without any upper bound checks. Standard simulation engines impose sensible maximum limits (e.g., 100,000 paths) and return errors when requested to exceed them.

✅ **Acceptance Criteria:**
- The `MonteCarloProjector` must not crash due to memory allocation failures when given extremely large path counts.
- The system must impose a reasonable upper limit on the number of paths and return a clean error if the requested number exceeds this bound.
- Existing Monte Carlo tests and chaos tests must pass.

🚫 **Out of Scope:**
- Transitioning the simulation engine to stream results to disk to support infinitely large path counts.
