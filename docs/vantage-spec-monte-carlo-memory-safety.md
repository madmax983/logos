# 🔭 Vantage: Spec for Monte Carlo Memory Safety

👤 **User Story:**
"As a user running Monte Carlo simulations, I want the system to safely handle massive simulation path requests without crashing, so that an accidental typo or extreme edge case does not crash the application due to out-of-memory errors."

🤔 **So What? (Business Problem):**
The Monte Carlo simulation allocates memory based on user input. If a user requests an absurdly high number of simulation paths (like 4 billion), it attempts to allocate gigabytes or terabytes of memory immediately, crashing the system and destroying the user session. Resilient software must clamp unreasonable memory allocation requests to safe limits to remain stable and trustworthy.

📈 **Metric Definition:**
Success = 0 system crashes or out-of-memory panics when the user requests an extremely high number of simulation paths (e.g., `u32::MAX`).

✅ **Acceptance Criteria:**
- The Monte Carlo simulator must not crash or panic when extremely large path counts are requested.
- The system must explicitly bound the maximum allowable simulation paths to a safe architectural limit.
- If the limit is exceeded, the system should gracefully clamp to the maximum safe limit or return an error, preventing out-of-memory panics.

🚫 **Out of Scope:**
- Transitioning to streaming processing models or out-of-core (disk-based) simulation memory.
- Dynamic detection of available physical RAM to adjust limits.
