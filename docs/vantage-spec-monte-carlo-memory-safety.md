# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
"As a user running probabilistic financial projections, I want the Monte Carlo simulator to safely handle requests for an enormous number of simulation paths without exhausting system memory, so that the application remains stable."

🤔 **So What?**
Simulation tools that accept user-defined parameters for iteration counts are vulnerable to memory exhaustion if extreme values are provided. If a user requests billions of paths, attempting to allocate memory for all of them at once will crash the system. A reliable product must protect itself against unreasonable resource requests to maintain stability and protect the host environment.

🎯 **Metric Definition:**
Success = 0 system crashes or out-of-memory errors when users request an impossibly large number of simulation paths. The system must gracefully reject unreasonable requests with a clear error message.

🔎 **Gap Analysis:**
Currently, the simulator directly allocates memory based on the number of requested simulation paths without bounding or streaming the calculation. When given an astronomically high path count, this results in a request for more memory than the system possesses, causing a hard abort. A robust simulation engine must either stream its aggregations to avoid massive allocations or enforce a strict upper limit on the number of concurrent paths.

✅ **Acceptance Criteria:**
- The Monte Carlo simulator must not crash or run out of memory when a user requests an extremely large number of simulation paths.
- The system must return a clear, user-friendly error if the requested number of paths exceeds a safe operational limit.

🚫 **Out of Scope:**
- Redesigning the simulator to write paths to disk or a database (we will simply bound the memory or stream aggregations).
- Distributed computing implementations for massive path simulations.
