# 🔭 Vantage: Spec for Tax Loss Harvester Resilience

## 👤 **User Story:**
As an investor holding taxable accounts, I want the system to safely calculate tax loss harvesting opportunities across all my assets, even those with extremely high valuations or significant market devaluations, so that the software remains stable and reliable during volatile market conditions.

## 🤔 **So What?**
During periods of high volatility or when tracking exceptionally large asset portfolios, extreme differences between the original purchase price and current market value can occur. If the application crashes when calculating these differences, users lose trust in the tool exactly when they need it most—during critical financial planning sessions. Ensuring robust calculations prevents data loss and maintains user confidence.

## 📈 **Metric Definition:**
Success = 0 application crashes when calculating tax loss harvesting opportunities for assets, regardless of how large the initial cost basis is or how steeply the current market price has fallen.

## 🔍 **Gap Analysis:**
The current tax loss harvester correctly identifies basic unrealized losses but struggles with extreme edge cases. When encountering an asset with an exceptionally high cost basis paired with a significant drop in market value (e.g., an asset becoming completely worthless), the underlying mathematical calculation fails, causing the entire system to crash instead of safely bounding the calculation or reporting the maximum possible loss.

## ✅ **Acceptance Criteria:**
- The system must safely compute the difference between any valid purchase price and current market value without crashing.
- If a calculated loss exceeds the maximum representable financial value, the system should gracefully cap the loss at the maximum safe value or return a structured error message to the user, rather than abruptly shutting down.
- The tax loss harvesting report must continue to generate successfully for all other stable assets even if one specific asset contains anomalous pricing data.

## 🚫 **Out of Scope:**
- Handling wash sale rule violations.
- Real-time execution of tax loss harvesting trades.
- Automatically correcting erroneous user-inputted market prices.