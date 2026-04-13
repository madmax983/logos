# 🔭 Vantage: Spec for Volatile Market Backtesting

## 👤 User Story
As a Trader, I want to backtest against volatile markets, so that I can evaluate the resilience of my portfolio.

## 🤔 So What? (Business Problem)
Standard historical backtesting assumes normal market conditions. However, real-world trading often involves sudden, volatile shifts. Providing a tool to explicitly test against volatile periods helps users uncover hidden risks, reducing unexpected losses and building trust in our platform's predictive capabilities. A resilient portfolio is a confident user.

## 🎯 Metric Definition
- Success Criteria: Users can define and execute a volatile backtest scenario.
- Performance: Backtest over a 10-year period completes in < 2 seconds.

## 🔍 Gap Analysis
- Current State: Backtesting relies on average historical returns.
- Competitors: Most standard tools offer basic backtesting; advanced tools offer Monte Carlo but are hard to configure.
- Our Edge: Simple, out-of-the-box volatile scenarios integrated directly with user portfolio data.

## ✅ Acceptance Criteria
- Must handle NaN data without panicking.
- Must output a CSV report.

## 🚫 Out of Scope
- Real-time execution (Phase 2).
