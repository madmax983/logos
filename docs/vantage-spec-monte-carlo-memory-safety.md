# 🔭 Vantage: Spec for Monte Carlo Memory Safety

👤 **User Story:**
As a user running Monte Carlo simulations, I want the system to safely handle unreasonably large requests for simulation paths, so that my application doesn't crash from memory exhaustion.

🤔 **So What? (Business Problem):**
The simulator currently panics when passing extreme path counts because it attempts a massive memory allocation. A financial tool should gracefully validate user inputs and return an error rather than requesting massive amounts of RAM and crashing the process. This degrades user trust and system stability.

📈 **Metric Definition:**
- Success = 0 memory allocation panics when passing maximum possible path counts to the simulator. The system should return an explicit error.

🔍 **Gap Analysis:**
The current implementation assumes the user will only provide reasonable values for path counts and allocates memory without any upper bounds checking.

✅ **Acceptance Criteria:**
- The system must validate the requested number of Monte Carlo paths against a reasonable maximum threshold.
- If the requested paths exceed the limit, the system must return a clear, user-facing error instead of attempting the allocation.
- The simulator must pass resilience tests against extreme inputs without panicking.

🚫 **Out of Scope:**
- Implementing disk-backed streaming for infinite path simulations.
