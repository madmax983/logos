# Statement Fetch Ops

## Current State

`logos-fetch` now owns statement-source config parsing, adapter execution, fetched artifact metadata, and persisted fetch-run status. The current branch has one fixture-backed institution adapter for `provident-credit-union` plus internal fixture adapters used by tests.

Live browser automation for M1, American Express, Robinhood, and a real Provident runner still sits in the next implementation slices. This doc describes the operator surface that already exists so the config shape, secret model, and recovery flow stay stable while the browser runners catch up.

## Config Location

`month autopilot` resolves fetch config in this order:

1. `LOGOS_FETCH_CONFIG_PATH`, when set
2. a sibling `statement-sources.toml` next to the ledger store path

Examples:

- Windows default ledger store: `%USERPROFILE%\\.logos\\ledger`
- Windows default fetch config: `%USERPROFILE%\\.logos\\statement-sources.toml`
- Explicit override: `LOGOS_FETCH_CONFIG_PATH=C:\\Users\\markm\\logos\\statement-sources.toml`

If `LOGOS_FETCH_CONFIG_PATH` is set and the file does not exist, autopilot fails immediately instead of silently pretending fetch is disabled.

Secret resolution uses the local 1Password CLI executable. By default Logos invokes `op`. Override that path with `LOGOS_FETCH_OP_BIN` when Task Scheduler or a nonstandard install location would otherwise miss it.

## Config Schema

```toml
[[sources]]
source_id = "pcu:checking"
institution_id = "provident-credit-union"
ledger_account = "assets:checking"
format_preference = ["pdf"]
username_secret_ref = "op://logos/provident/username"
password_secret_ref = "op://logos/provident/password"
totp_secret_ref = "op://logos/provident/totp"
```

Field rules:

- `source_id`: stable identifier for one institution account
- `institution_id`: adapter key, for example `provident-credit-union`
- `ledger_account`: account scope that `month autopilot` reconciles
- `format_preference`: preferred artifact order; current autopilot only imports fetched PDFs
- `*_secret_ref`: 1Password references only; raw secrets are rejected by config parsing

Multiple sources may point at the same `ledger_account`. If at least one source fetches usable statement evidence for that month, autopilot can continue while the failing source is persisted as `needs_attention`.

## 1Password Setup

Keep usernames, passwords, and TOTP secrets in 1Password items and store only `op://...` references in `statement-sources.toml`.

Recommended layout:

- one vault for Logos automation secrets
- one item per institution account
- separate fields for username, password, and TOTP seed/code material

Useful checks before scheduling anything:

- `op read "op://logos/provident/username"`
- `op read "op://logos/provident/password"`

If `op` is not on `PATH`, set `LOGOS_FETCH_OP_BIN` to the full executable path before running autopilot.

If a future browser runner executes under Task Scheduler, it must run as the same Windows user that can access the relevant 1Password account and vault session.

## TOTP Migration From Google Authenticator

Logos does not extract TOTP seeds from Google Authenticator. If an institution allows MFA re-enrollment, move the TOTP seed into 1Password and verify login there before deleting the old factor.

Recommended migration order:

1. add the new authenticator target in 1Password
2. confirm a fresh login works with the 1Password-generated code
3. save backup codes outside the browser profile
4. remove the old Google Authenticator factor only after the new path is proven

If the seed cannot be moved, treat that source as manual-recovery-only. In practice that means the adapter should surface `needs_attention` and the operator reruns autopilot after handling MFA interactively.

## Running Autopilot

Example command:

```powershell
ledger month autopilot --month 2026-02 --checking-account assets:checking --confirm-close
```

Autopilot behavior:

- uses explicit balances when they are provided
- otherwise attempts configured statement fetch
- imports a fetched PDF artifact when one exists
- persists each fetch run whether it succeeded, needed attention, or failed
- closes the month scope only after reconciliation succeeds

If one source succeeds and another source for the same account lands in `needs_attention`, the month can still close because statement evidence exists for that scope. The attention state is preserved for follow-up instead of being swallowed.

## Windows Task Scheduler Notes

For scheduled execution:

- run the task as the same Windows user that owns the ledger store path
- set `LOGOS_DB_PATH` and, when needed, `LOGOS_FETCH_CONFIG_PATH`
- set `LOGOS_FETCH_OP_BIN` when 1Password CLI is installed outside `PATH`
- capture stdout/stderr to a log file so failed fetch runs are not a séance
- keep any future browser profile, 1Password session strategy, and automation runtime under that same user context

Until live browser adapters land, scheduled runs mainly exercise config loading, persisted fetch-run tracking, and fixture-backed adapter behavior.

## Recovering From `needs_attention`

Use the fetch CLI first:

```powershell
ledger fetch list-runs --month 2026-02 --checking-account assets:checking
ledger fetch show-run --run-id fetch-17
```

Recovery loop:

1. inspect the failing source and its `error_summary`
2. fix the underlying issue: bad secret ref, expired password, MFA re-prompt, or runner breakage
3. rerun `ledger month autopilot ... --confirm-close`
4. verify that a new fetch run for the source moved to `downloaded`, `imported`, or `no_new_statement`

If no source for the account produced usable statement metadata, autopilot blocks close and forces manual intervention instead of inventing balances.
