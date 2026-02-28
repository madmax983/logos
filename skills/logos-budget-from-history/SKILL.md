---
name: logos-budget-from-history
description: Use when the user wants to derive a monthly budget from recent ledger history (for example 2-6 months), then review and apply budget targets by category.
---

# Logos Budget From History

## Overview

Guide the user through building budget targets from real spending history. Use a conservative baseline so one-off spikes do not dominate targets.

Ask one question per message. Do not batch questions.

## Flow

1. Scope the analysis window
- Ask for month range or number of prior months (default 3).
- Ask for target budget month (`YYYY-MM`).
- Ask for account scope and expense prefix (default `expenses:`).

2. Gather transaction history
- Pull historical expenses from ledger for the chosen window.
- Group by category (two-level taxonomy unless user says otherwise).
- Exclude transfers, debt principal moves, and known non-recurring events when user flags them.

3. Compute proposed category budgets
- For each category, compute:
mean, median, max, min, and count by month.
- Default recommendation:
`recommended = max(median, round(mean * 0.9))`
- If fewer than 2 months exist for a category, mark low-confidence.

4. Build final budget plan
- Ask whether to keep, raise, or lower each recommended category.
- Add explicit buffer line (for example `expenses:buffer`) when requested.
- Return total planned spend and compare against expected income.

5. Emit executable commands
- Produce paste-ready commands:
`cargo run -p logos-cli -- budget set --month <YYYY-MM> --budget-cents <i64> --expense-account-prefix <category-prefix>`
- Only emit write commands after explicit user confirmation.

## Guardrails

- Use integer cents only.
- Use absolute month keys (`YYYY-MM`).
- Never assume category exclusions without user approval.
- Always label low-confidence categories when history is sparse.
- Keep a list of assumptions in final output.
