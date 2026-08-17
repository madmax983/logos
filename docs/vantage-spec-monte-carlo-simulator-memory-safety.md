# 🔭 Vantage: Spec for Monte Carlo Simulator Memory Safety

👤 **User Story:**
As a user running long-term financial simulations, I want the Monte Carlo simulator to handle massive simulation path requests safely without crashing my machine, so that I can confidently experiment with different simulation parameters.

🤔 **So What?**
Financial simulations often allow users to dictate the number of randomized paths to calculate. If a user inputs an excessively large number, the system currently attempts to allocate all necessary memory at once. This leads to immediate system crashes due to memory exhaustion. A robust financial product must gracefully handle extreme inputs and provide clear boundaries, ensuring the tool remains reliable and trustworthy.

📈 **Metric Definition:**
Success = 0 system crashes or out-of-memory panics when users request an extreme or maximum possible number of simulation paths. The system must return a clear, user-friendly error about the exceeded limits.

🔎 **Gap Analysis:**
The current simulation engine blindly trusts the user's input for the number of paths and attempts to pre-allocate memory for all of them. Unlike professional financial software that imposes hard caps or streams calculations to prevent system lockups, our tool tries to fulfill mathematically impossible memory requests, leading to immediate failure.

✅ **Acceptance Criteria:**
- The Monte Carlo simulation engine must safely bound the maximum number of simulation paths.
- Requests exceeding the safe threshold must immediately return a descriptive error explaining the limitation, without attempting allocation.
- The system must not panic or crash due to memory allocation failures caused by user input.

🚫 **Out of Scope:**
- Implementing out-of-core or streaming Monte Carlo algorithms to support infinite path calculations.
- Adjusting the underlying statistical model or randomness generation mechanism.
