# 🔭 Vantage: Spec for Robust Reporting Engine

👤 **User Story:**
"As an enterprise user with high-value transactions or large aggregated historical balances, I want the reporting engine to accurately project my balance across large numbers, so that I can rely on the system without worrying about crashes or calculation failures during edge-case portfolio sizes."

💼 **Business Problem:**
The system currently crashes catastrophically (panicking on arithmetic overflow) when evaluating extreme account balances or large aggregate sums. Such fragility violates the core tenet of professional financial software. A billionaire entering their portfolio balance should not crash the reporting engine. Resilience against large numbers is critical to maintaining utility and user trust.

✅ **Acceptance Criteria:**
- **Success Metric:** The `project_register_balance_iter` function (and the broader register reporting pathway) never panics due to arithmetic overflow, even when processing maximum 64-bit integer values (`i64::MAX`).
- The system must use safe, saturating arithmetic (e.g., `saturating_add` or explicit bounding) when projecting running balances or aggregating sums.
- In the event that an account balance legitimately exceeds the mathematical bounds of a 64-bit integer, the system must cap the value cleanly rather than crashing the thread.
- The `havoc_proptest` for `logos-reporting` must consistently pass without the `#[should_panic]` attribute (or the test must be rewritten to assert safe saturation), demonstrating that the system handles edge-case integers without panicking.

🚫 **Out of Scope:**
- Upgrading the underlying `i64` precision to BigInt or 128-bit integers (saturating 64-bit logic is sufficient for the immediate robustness goal).
- Building an interactive UI to warn the user about theoretical limits (the goal is simply not to crash).
