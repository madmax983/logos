# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
"As a financial planner exploring probabilistic outcomes, I want the system to safely reject unreasonably large path requests (like 4 billion simulations) instead of crashing, so that the application remains responsive and reliable."

🤔 **So What?**
Monte Carlo simulations are memory-intensive. If a user accidentally or maliciously requests an astronomically high number of paths, the application attempts to allocate memory corresponding to that size up-front. This results in an immediate Out-Of-Memory (OOM) panic, killing the entire process. Protecting the memory boundaries ensures application uptime and prevents denial-of-service through resource exhaustion.

🎯 **Metric Definition:**
Success = The Monte Carlo Simulator safely rejects extreme path values without attempting allocation, returning a graceful error instead of an OOM panic.

🔍 **Gap Analysis:**
Currently, the system directly accepts the number of paths requested and passes it to internal capacity allocations. There are no upper bounds or sanity checks on this input, leaving the system vulnerable to memory allocation panics.

✅ **Acceptance Criteria:**
- The system must enforce a reasonable upper limit on the number of Monte Carlo paths requested (e.g., a cap of 1,000,000 paths or similar depending on reasonable usage).
- If the requested number of paths exceeds this maximum limit, the system must immediately return an explicit, descriptive error (e.g., `TooManyPaths` or `ResourceLimitExceeded`) without attempting any memory allocation.
- The system must not panic on memory allocation when an excessive path count is provided.
- All tests for the Monte Carlo Simulator must pass, and new tests should verify that excessive path requests result in a graceful error, not a panic.

🚫 **Out of Scope:**
- Chunking or streaming the Monte Carlo results to disk to support infinitely large simulations. We will simply enforce a sane upper memory bound for now.
- Fixing memory issues or overflow in other parts of the financial engine (they will have separate specs).
