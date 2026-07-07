# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

## 👤 User Story:
"As a long-term investor, I want to run extensive Monte Carlo simulations, so that I can understand risk without the system crashing due to out-of-memory errors when I request a massive number of paths."

## 🤔 So What?
Users often want to "stress test" scenarios by dialing up parameters. If requesting an extremely high number of simulation paths causes the application to aggressively allocate memory until the operating system kills it (an OOM crash), trust is destroyed. The system needs to either cap the memory usage or fail fast with a polite boundary error instead of crashing the host environment.

## 🎯 Metric Definition:
Success = 0 OOM crashes or panics when running `MonteCarloProjector::run` with `u32::MAX` paths. The system must gracefully return an error or enforce a safe upper limit.

## 🔎 Gap Analysis:
The current `MonteCarloProjector` directly attempts to allocate a vector with capacity equal to the requested number of paths. If the user passes `u32::MAX`, it tries to allocate over 34GB of RAM instantly, causing a hard panic on most systems.

## ✅ Acceptance Criteria:
- The Monte Carlo simulation must not panic due to memory allocation failure when given massive path requests.
- The system must implement a maximum safe threshold for simulation paths (e.g., 1,000,000) or stream results without allocating massive contiguous memory blocks.
- The system should return a clear, user-friendly error message if the requested path count exceeds the safe limits.

## 🚫 Out of Scope:
- Distributing the simulation across a cluster or writing results to disk to allow infinite paths.
