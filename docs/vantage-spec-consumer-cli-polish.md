# Vantage: Spec for Consumer-Grade CLI Polish

User story:

As a new user, I want the supported startup path to be obvious and the default console output to be readable, so that I can begin tracking finances without reverse-engineering storage internals.

So what?

Onboarding dies when the setup story is ambiguous. The CLI should tell the truth: Logos is Postgres-backed, migrations are explicit, and the happy path is `docker compose up -d db`, set `DATABASE_URL`, then `ledger db migrate`. The user should not be hit with internals cosplay while doing ordinary commands.

Metric definition:

- Success: a fresh install can follow the documented local Postgres path and run `cargo run -p logos-cli -- db migrate`
- Success: default non-debug stdout for standard workflows avoids internal storage implementation jargon

Acceptance criteria:

- Must document one supported local startup path for storage-backed commands
- Must fail fast with a clear message when `DATABASE_URL` is missing or migrations are pending
- Must hide storage implementation details unless verbose diagnostics are explicitly requested

Out of scope:

- Replacing the chosen Postgres architecture
- GUI onboarding flows
- Magical auto-migrations at runtime startup
