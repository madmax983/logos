# 🔭 Vantage: Spec for Subscription Fatigue Analyzer

## 👤 User Story
As a user burdened by numerous monthly subscriptions, I want a single report that combines the detection of recurring expenses with their long-term opportunity costs, so that I can see the true financial impact of my subscriptions in one place.

## 🤔 So What? (Business Problem)
Identifying subscriptions is only half the battle; understanding their true cost motivates action. By combining recurrence detection with opportunity cost analysis, we create a high-impact "Subscription Fatigue" report that drives users to optimize their cashflow.

## 📈 Metric Definition
Success = The system automatically detects recurring subscriptions and calculates the total monthly cost alongside the aggregated long-term opportunity cost.

## 🔍 Gap Analysis
We have isolated engines for detecting recurring templates and calculating opportunity cost, but users currently have to bridge these concepts manually. A unified analyzer and report format solves this UX gap.

## ✅ Acceptance Criteria
- Must combine recurrence detection and opportunity cost analysis in a single run.
- Must accept parameters for minimum occurrences, annual return percentage, and projection years.
- Must output a comprehensive report listing each identified subscription with its individual monthly and future costs.
- Must sum the total monthly cost across all identified subscriptions.
- Must sum the total opportunity cost across all identified subscriptions.

## 🚫 Out of Scope
- Alerting users when a subscription increases in price.
- Connecting to external services to cancel subscriptions.
