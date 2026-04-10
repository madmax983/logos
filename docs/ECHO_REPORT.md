# Echo: Getting Started compile break resolved

The old prototype storage setup used a developer-local absolute path and made the repo fail to compile for anyone who did not already have the same machine layout. That was the whole goblin pile behind the original onboarding complaint.

Current operator contract:

- `cargo run -p logos-cli -- help` works from a fresh clone without external local path hacks
- storage-backed commands require `DATABASE_URL`
- local development uses `docker compose up -d db` followed by `cargo run -p logos-cli -- db migrate`

The repo now treats Postgres as the only production store, keeps migrations explicit, and no longer relies on unpublished local storage dependencies.

## DX Audit Update
The issues raised regarding hardcoded aletheia manifest paths and confusing internal database jargon output during CLI startup have been fully resolved by the 'Postgres+Diesel storage cutover'. The codebase no longer contains the Aletheia server infrastructure or the associated logging.
