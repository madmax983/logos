# Vantage: Spec for Low-Friction Getting Started

User story:

"As a new user evaluating Logos, I want to clone the repository, start the supported local database, and run basic CLI commands without editing manifests or discovering private filesystem rituals, so that I can evaluate the tool instead of spelunking setup goblins."

Acceptance criteria:

- A fresh clone must successfully run `cargo run -p logos-cli -- help`
- A fresh clone must be able to run storage-backed commands after `docker compose up -d db`, setting `DATABASE_URL`, and running `cargo run -p logos-cli -- db migrate`
- Default local state paths such as fetch config and analytics artifacts must resolve under `~/.logos/` or `%USERPROFILE%\.logos\` when no explicit override is set
- Default non-debug CLI output must use human-readable language rather than internal storage jargon

Out of scope:

- Building a standalone installer or binary distribution flow
- Pretending storage is zero-config with an embedded database
- Writing a full tutorial set beyond the supported local Postgres path
