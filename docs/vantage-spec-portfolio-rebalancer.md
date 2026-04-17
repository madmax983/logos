# 🔭 Vantage: Spec for Portfolio Rebalancer

👤 **User Story:**
"As an investor, I want to define target percentage allocations for my assets and have the system automatically calculate the exact trades needed to return my portfolio to those targets, so that I can easily maintain my desired risk profile without manual math."

🤔 **So What?**
Over time, market movements cause asset allocations to drift away from their intended targets, altering the risk profile. Manually calculating the necessary buys and sells to restore balance is tedious and error-prone, especially across multiple accounts. Automating this ensures users maintain disciplined asset allocation efficiently.

🎯 **Metric Definition:**
Success = The system correctly generates the exact balancing actions required to reach the target allocation percentages, exactly balancing the total portfolio value. Target allocations must always total exactly 100%.

🔍 **Gap Analysis:**
Currently, users must manually calculate percentage differences across their accounts and manually craft adjusting entries. Most standard consumer tools only show the current allocation, leaving the "how to fix it" math entirely to the user.

✅ **Acceptance Criteria:**
- The user must be able to specify a list of target assets and their desired percentages.
- The target percentages must sum to exactly 100%. If they do not, the system must clearly report an error to the user.
- The system must analyze the current value of the portfolio and output the precise balancing actions required to reach the targets.
- The total value of the portfolio must remain exactly the same before and after the rebalancing calculation; no money can be created or destroyed.

🚫 **Out of Scope:**
- Automatic execution of trades at brokerages.
- Factoring in tax implications (e.g., short-term vs long-term capital gains) of the required sales.
- Real-time market data integration.
