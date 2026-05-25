# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Allocation Safety

## 👤 User Story
As a user running financial projections, I want the Monte Carlo simulator to gracefully reject unreasonably large simulation parameters, so that my system does not crash or run out of memory when I make a typo in the inputs.

## 🤔 So What? (Business Problem)
Currently, passing a massive number of paths (like `u32::MAX`) to the Monte Carlo simulator causes the system to attempt a multi-gigabyte memory allocation, crashing the application instantly. Financial tools must be robust and reliable. When the application panics due to bad input, it degrades user trust and creates a poor developer/user experience. Preventing these crashes ensures our system behaves predictably under all conditions.

## 🎯 Metric Definition
- Success = Running the Monte Carlo simulator with `u32::MAX` paths returns a clear, helpful error message rather than a memory allocation panic.
- Quality = 0 memory allocation panics triggered during fuzz testing or extreme edge-case usage.

## 🔍 Gap Analysis
Standard robust applications validate input boundaries before allocating memory. Our simulator currently trusts the user input blindly. We need to define a safe upper bound for simulation paths that balances statistical significance with hardware limits.

## ✅ Acceptance Criteria
- The system must enforce a maximum limit on the number of paths the Monte Carlo simulator can process (e.g., 100,000 or 1,000,000 paths).
- If a user requests more paths than the maximum limit, the system must return a clear, human-readable error explaining the limit.
- The system must never panic due to excessive memory allocation requests from user input.

## 🚫 Out of Scope
- Implementing streaming calculations to support infinite paths (we will enforce a hard limit for now).
- Optimizing the underlying memory footprint of a single path.
