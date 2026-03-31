# ADR 0002: Postgres Runtime as CLI Default

Date: 2026-02-27  
Status: Accepted

## Context

`logos` is a local personal-finance CLI/TUI. The primary workflow needs durable local writes, predictable CI behavior, and adoption-friendly infrastructure. The earlier embedded temporal-storage experiment added onboarding friction and split the storage story away from the rest of the Rust/Postgres ecosystem.

## Decision

1. `logos-cli` uses Postgres through `logos-store-pg` as the default and only production runtime mode.
2. `DATABASE_URL` is required for runtime startup.
3. Schema changes are explicit; runtime startup fails fast when pending migrations exist, and operators must run `ledger db migrate`.
4. Journal writes stay append-only with correction links; historical reads use transaction `effective_at` plus `recorded_at` semantics.

## Consequences

Positive:

- Local development and CI can use standard Postgres workflows and tooling.
- Persistence survives process restarts by default.
- The backend is understandable to more contributors and operators than a custom embedded temporal store.

Tradeoffs:

- Operators now manage a Postgres service instead of a local embedded file path.
- Runtime startup depends on explicit migration discipline.

## Operational Notes

- Use standard Postgres backup and restore procedures.
- For local development, `docker compose up -d db` is the supported convenience path.
- Use `ledger db status` and `ledger db migrate` to inspect and apply schema changes.
