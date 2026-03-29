# 🔭 Vantage: Spec for Robust Register Balance

👤 **User Story:**
"As a user managing large or aggregated financial portfolios, I want the reporting engine to gracefully handle extremely large numbers, so that my dashboard does not crash unexpectedly during balance projection or ledger analysis."

🤔 **So What?**
Financial software must inspire absolute trust. Panicking or crashing on large numeric inputs (e.g., arithmetic overflows) shatters user confidence and makes the tool unreliable for high-net-worth individuals, institutional tracking, or extreme edge cases (like hyper-inflationary currencies). Silence or errors are acceptable; violent crashes are not.

🎯 **Metric Definition:**
Success = 0 panics in the reporting engine when calculating register balances, regardless of input size.

🔍 **Gap Analysis:**
Standard libraries often panic on overflow in debug mode or wrap silently in release mode (which corrupts financial data). We need explicit bounds checking or saturating/checked math that returns a domain error rather than crashing the process.

✅ **Acceptance Criteria:**
- The `project_register_balance_iter` function (and similar reporting aggregations) must return a graceful error (e.g., `Result::Err`) instead of panicking when an arithmetic overflow occurs.
- Financial balances must not be silently capped or wrapped (e.g., do not use `saturating_add` to blindly cap at `i64::MAX` as this corrupts ledger data).
- The CLI/TUI must display a user-friendly error message if a calculation exceeds the maximum supported bounds.

🚫 **Out of Scope:**
- Upgrading all internal `i64` representations to arbitrary-precision `BigInt` (we will stick to explicit error handling for `i64` bounds first).
- Implementing new financial features; this is strictly about hardening existing calculations.
