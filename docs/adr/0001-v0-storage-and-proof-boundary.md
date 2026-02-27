# ADR 0001: v0 Storage and Proof Boundary

Date: 2026-02-23  
Status: Accepted

## Context

`logos` v0 is a personal dogfooding finance tool that prioritizes:

- strict double-entry correctness
- append-only correction semantics
- budget + RSU policy invariants
- rapid iteration on CLI/TUI workflows

The v0 design calls for Aletheia-first storage and a Verus-backed proof boundary (`logos-proof`) for critical invariants.

## Decision

1. Use `logos-store-aletheia` as the storage contract layer for journal writes, correction edges, and report reads.
2. Keep invariant-heavy logic in `logos-core`; treat adapters/IO as unverified glue.
3. Encode proof spines in `logos-proof` for:
   - transaction balance properties
   - correction no-self-cycle properties
   - budget rollover conservation
   - RSU policy allocation/tier properties

## Consequences

Positive:

- Domain invariants remain centralized and testable.
- Storage adapter can evolve without destabilizing core rules now that embedded durable mode is the default runtime path.
- Verus proof artifacts document mathematical intent and expected safety properties.

Tradeoffs:

- Proofs still cover spine lemmas, not full end-to-end adapter behavior.
- CLI and store behavior now include local durable persistence and bi-temporal reads, which increases adapter complexity and test scope.

## Follow-up

- Keep strengthening regression coverage for month-windowed reporting and budget-target persistence.
- Add tighter coupling between proven invariants and runtime representations where feasible.
