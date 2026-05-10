# 🔭 Vantage: Spec for Monte Carlo Projector Memory Safety

👤 **User Story:**
As a user running Monte Carlo simulations, I want the system to restrict the maximum number of simulation paths I can request, so that I don't accidentally crash the application by running out of memory.

🤔 **So What? (Business Problem):**
Allocating memory based directly on unconstrained user input is a critical reliability risk. If a user requests billions of paths, the application attempts to allocate gigabytes of RAM instantly, causing the OS to kill the process (`SIGABRT`). A resilient system must protect itself against user-induced resource exhaustion.

🎯 **Metric Definition:**
Success = 0 memory allocation panics when requesting `u32::MAX` paths.

🔍 **Gap Analysis:**
The current `MonteCarloProjector` directly passes the requested path count to `Vec::with_capacity()`. There is no upper bound enforced, meaning extreme inputs bypass normal operation and trigger hard memory crashes.

✅ **Acceptance Criteria:**
- The `MonteCarloProjector` must enforce a safe maximum limit on the number of simulation paths.
- External inputs that drive memory allocations must be clamped to this safe maximum.
- The application must not crash with a memory allocation failure when given `u32::MAX` as the path count.

🚫 **Out of Scope:**
- Changing the simulation logic.
- Streaming results to disk to support infinite paths.
