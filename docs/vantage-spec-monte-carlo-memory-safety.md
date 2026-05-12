# 🔭 Vantage: Spec for Monte Carlo Memory Safety

## 👤 User Story
As a user running Monte Carlo simulations, I want the system to safely handle extreme simulation path requests, so that my application doesn't crash from out-of-memory errors when analyzing my portfolio.

## 🤔 So What? (Business Problem)
Currently, requesting a massive number of paths (like `u32::MAX`) causes the simulator to attempt to allocate gigabytes of memory at once via `Vec::with_capacity(paths as usize)`, resulting in a fatal out-of-memory (OOM) `SIGABRT` crash. This makes the application brittle and degrades user trust. A robust financial engine must enforce sane resource boundaries to maintain stability.

## 📈 Metric Definition
- Success = 0 memory allocation crashes when requesting an arbitrarily large number of simulation paths.
- The system must gracefully return a domain error rather than terminating the process.

## 🔍 Gap Analysis
The `MonteCarloProjector` blindly trusts user input and passes it directly to `Vec::with_capacity`. There is no bounds checking or maximum limit enforced on the input parameter.

## ✅ Acceptance Criteria
- Must enforce a reasonable upper bound on the number of simulation paths (e.g., 100,000 or 1,000,000).
- Must return a clear, structured domain error if the requested paths exceed this limit.
- Must not crash or panic from memory exhaustion when provided with massive inputs.

## 🚫 Out of Scope
- Implementing chunked or streaming Monte Carlo generation to bypass memory limits.
- Rewriting the PRNG logic.
