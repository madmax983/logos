# 🔭 Vantage: Spec for Opportunity Cost Analyzer

## 👤 User Story
As a user tracking my recurring expenses, I want to see the long-term opportunity cost of my subscriptions if that money had been invested instead, so that I can make informed decisions about whether to cancel non-essential services.

## 🤔 So What? (Business Problem)
Users often underestimate the true cost of small recurring expenses. By providing an "Opportunity Cost Analyzer," the tool moves beyond simple expense tracking to active wealth building strategy. This increases user engagement by framing expense cuts as investment gains.

## 📈 Metric Definition
Success = The system calculates the projected future value of a recurring expense over a configurable number of years using a specified annual return rate.

## 🔍 Gap Analysis
Currently, users can see their monthly total for subscriptions, but not the compounded long-term effect of that spending. Exposing this through a dedicated command or integrated report closes the gap between expense tracking and investment planning.

## ✅ Acceptance Criteria
- Must take an expected annual return percentage (e.g., 7.0%).
- Must take a projection horizon in years.
- Must calculate the future value of a recurring monthly payment using the standard future value of a series formula.
- Must output the total monthly cost and the projected future value.
- Must handle zero-return edge cases gracefully.

## 🚫 Out of Scope
- Integrating live market return data (uses user-provided static return rates).
- Complex tax implications on investment returns.
