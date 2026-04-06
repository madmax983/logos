# 🔭 Vantage: Spec for Robust Reporting Bounds

👤 **User Story:**
As a User with large financial transactions or extensive theoretical projections, I want the reporting engine to gracefully handle extremely large numbers (like massive RSU vests or long-term FIRE compound interest), so that the application doesn't crash or panic when I stress-test my financial plans.

🤔 **So What? (Business Problem):**
The application currently panics and crashes when reporting values exceed the 64-bit integer limit (e.g., during arithmetic overflow in `project_rsu_forecast_summary`). Crashing the application destroys user trust, especially for users who are using the tool for long-term compound interest modeling or theoretical stress testing. A resilient application should handle extreme boundary values gracefully (e.g., by saturating at maximum bounds or returning a handled error) rather than crashing the entire process. Reliability is a core utility.

📈 **Metric Definition:**
- Success = 0 application panics due to arithmetic overflow during reporting and projection computations.

🔍 **Gap Analysis:**
- Current State: Arithmetic operations (like `.sum::<i64>()`) are unguarded and crash the program in debug/test or wrap around silently in release mode.
- Competitors: Standard financial tools handle overflow gracefully or have significantly higher numerical limits.
- Our Edge: We want an application that is mathematically robust and impossible to crash via user input or projection bounds.

✅ **Acceptance Criteria:**
- The system must explicitly handle arithmetic overflows in all reporting projections (e.g., RSU forecasting, net worth projections).
- When an operation would overflow the maximum supported integer size (e.g., `i64::MAX`), the system must either explicitly saturate at the maximum boundary or return a structured application error, never panicking.
- The system must pass fuzz testing and extreme bounds testing (e.g., supplying `i64::MAX` to inputs) without crashing.

🚫 **Out of Scope:**
- Upgrading all internal representations from `i64` to `i128` or `BigInt` (we just need graceful handling, not infinite precision).
- Modifying the core ledger transaction validation (this spec focuses on the reporting/projection layer).
