# 🔭 Vantage: Spec for Monte Carlo Capacity Safety

👤 **User Story:**
"As a long-term investor, I want to project my future net worth with a Monte Carlo simulator safely, so that I can explore extreme edge cases or high path counts without crashing my system."

🤔 **So What?**
Financial simulation tools must gracefully handle extreme or absurd inputs. Currently, passing an excessively high path count to the Monte Carlo projector attempts to allocate massive amounts of system memory all at once, leading to a catastrophic system abort (SIGABRT/OOM). When a tool panics and crashes the whole system rather than refusing the bad input, it severely erodes user trust and prevents the user from relying on the tool for stress testing.

🎯 **Metric Definition:**
Success = 0 system crashes or out-of-memory aborts when the Monte Carlo projector is provided with excessively large path counts. The simulation must gracefully reject the input or return a clear bounds error without crashing.

🔍 **Gap Analysis:**
The current Monte Carlo simulator directly allocates an unbounded array based strictly on the user-provided number of simulation paths. This leaves the system completely vulnerable to capacity limits. A robust engine must define safe operational constraints or batch operations to prevent users from accidentally demanding more RAM than the system can provide.

✅ **Acceptance Criteria:**
- The Monte Carlo simulator must not crash or trigger a SIGABRT when provided with massive path counts (e.g., `u32::MAX`).
- The system must explicitly validate the path count before memory allocation.
- If the path count exceeds a safe threshold, the system must return a clear boundary error instead of attempting the simulation.
- Existing crash tests (like the Havoc OOM test) must fail gracefully instead of aborting the process.

🚫 **Out of Scope:**
- Streaming Monte Carlo results to disk to allow infinite path counts.
- Cloud-based distributed path calculation.