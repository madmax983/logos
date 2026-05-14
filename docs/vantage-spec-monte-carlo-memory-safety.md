# 🔭 Vantage: Spec for Monte Carlo Memory Safety

👤 **User Story:**
"As a user running probabilistic simulations, I want the system to safely reject or cap impossibly large simulation path requests, so that my system does not crash or run out of memory."

🤔 **So What?**
A simulation engine must protect the host system's resources. An unguarded request for billions of paths will immediately crash the application via Out-Of-Memory (OOM) panic. We need to enforce safe boundaries to guarantee application stability and predictable performance.

🎯 **Metric Definition:**
Success = 0 OOM panics during Monte Carlo projections. The system gracefully returns an error or clamps the maximum allowed paths when an excessive number of paths (e.g., `u32::MAX`) is requested.

🔎 **Gap Analysis:**
Currently, `MonteCarloProjector::run` blindly trusts the `paths` parameter and attempts to allocate memory up front via `Vec::with_capacity(paths as usize)`. Standard simulation tools enforce a maximum path count or return a capacity error.

✅ **Acceptance Criteria:**
- The `MonteCarloProjector::run` function must not panic or cause an OOM abort when requested to run extremely large numbers of paths.
- The path count must be safely constrained, either by returning a structured domain error (e.g., exceeding maximum capacity) or clamping to a safe upper bound.
- All existing tests, including chaos tests that currently trigger the `SIGABRT` via `u32::MAX` paths, must pass safely.

🚫 **Out of Scope:**
- Distributed/cloud-based Monte Carlo processing.
- Streaming paths to disk to avoid memory limits entirely.