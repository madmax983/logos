# 🔭 Vantage: Spec for Investment Simulator Resource Safety

👤 **User Story:**
As a user running probabilistic investment simulations, I want the system to cleanly reject requests for an excessive number of simulation paths, so that my device does not run out of memory and crash the application.

🤔 **So What?**
The investment simulator calculates many potential future outcomes. If a user inputs an astronomically high number of simulations to run, the system attempts to reserve memory for all of them at once. This leads to system instability and abrupt crashes due to memory exhaustion. A reliable product must enforce safe operational limits to protect the user's environment and maintain trust.

📈 **Metric Definition:**
- Success = 0 system crashes or out-of-memory errors when users request an extreme number of simulation paths.
- The system should gracefully return an error if the requested number of simulations exceeds safe resource bounds.

🔎 **Gap Analysis:**
Currently, the simulation engine attempts to fulfill any requested volume of simulation paths, regardless of how large the number is, directly requesting system memory proportional to that number. This blind trust in the input size leaves the application vulnerable to immediate resource exhaustion and termination.

✅ **Acceptance Criteria:**
- The investment simulator must not crash or run out of memory when a user requests a massive number of simulation paths.
- The system must enforce a maximum safe limit on the number of simulation paths it will attempt to process.
- Requests exceeding this limit must be gracefully rejected or clamped to a safe maximum, returning a clear error if rejected.

🚫 **Out of Scope:**
- Streaming simulation results to disk to allow for infinite paths.
- Distributed processing of simulations across multiple machines.
