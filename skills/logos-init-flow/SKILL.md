---
name: logos-init-flow
description: Use when onboarding a new logos ledger and the user wants a guided, question-by-question setup for accounts, opening balances, and first monthly budget targets.
---

# Logos Init Flow

## Overview

Run a conversational setup wizard for first-time personal finance onboarding in this repo. Ask one question per message. Do not batch questions.

The outcome is a ready-to-run command list that initializes accounts and first budget settings with explicit timestamps.

## Flow

1. Confirm setup context
- Ask for setup timestamp in local time (`YYYY-MM-DDTHH:MM:SS+/-HH:MM`), default now.
- Ask for default month scope (`YYYY-MM`) and checking account name (default `assets:checking`).

2. Build chart of accounts
- Collect active accounts the user actually uses:
`assets:*`, `liabilities:*`, `income:*`, `expenses:*`, plus `equity:opening-balances`.
- Prefer two-level taxonomy for expense categories unless user requests deeper nesting.

3. Capture opening balances
- For each real account, collect opening balance in cents at the setup timestamp.
- Treat liabilities as amount owed.
- If any balance is unknown, mark it explicitly as unresolved and continue.

4. Generate initialization commands
- Use repo commands:
`cargo run -p logos-cli -- txn add ...`
`cargo run -p logos-cli -- budget set ...`
`cargo run -p logos-cli -- budget rsu-plan ...` (only if RSU workflow applies).
- For opening balances:
asset +balance: debit asset, credit `equity:opening-balances`
liability +balance: debit `equity:opening-balances`, credit liability

5. Finalize
- Return:
assumptions summary
paste-ready command block
unresolved items list
- Ask for confirmation before any write command is executed.

## Guardrails

- Use absolute dates and times, never relative words like "today" without restating exact date.
- Keep monetary values as integer cents in commands.
- Never invent balances, account names, or category mappings.
- If command flags are uncertain, inspect local CLI help before emitting final commands.
