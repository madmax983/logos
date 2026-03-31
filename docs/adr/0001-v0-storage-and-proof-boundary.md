# ADR 0001: v0 Storage and Proof Boundary

Date: 2026-02-23  
Status: Accepted

## Context

`logos` v0 is a personal finance tool that prioritizes:

- strict double-entry correctness
- append-only correction semantics
- budget + RSU policy invariants
- rapid iteration on CLI/TUI workflows

The storage boundary is now Postgres-first, with a Verus-backed proof boundary (`logos-proof`) for critical invariants.

## Decision

1. Use `logos-store` as the storage contract boundary and `logos-store-pg` as the Postgres/Diesel implementation.
2. Keep invariant-heavy logic in `logos-core`; treat adapters/IO as unverified glue.
3. Encode proof spines in `logos-proof` for:
   - transaction balance properties
   - correction no-self-cycle properties
   - budget rollover conservation
   - RSU policy allocation/tier properties

## Consequences

Positive:

- Domain invariants remain centralized and testable.
- Storage adapters can evolve without destabilizing core rules now that runtime behavior targets a neutral store contract.
- Verus proof artifacts document mathematical intent and expected safety properties.

Tradeoffs:

- Proofs still cover spine lemmas, not full end-to-end adapter behavior.
- CLI and store behavior include explicit migrations, durable Postgres persistence, and temporal journal reads, which increases adapter complexity and test scope.

## Follow-up

- Keep strengthening regression coverage for month-windowed reporting and budget-target persistence.
- Add tighter coupling between proven invariants and runtime representations where feasible.
