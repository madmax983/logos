## 2025-02-27 - [Debits and Credits Sign Convention]
**Confusion:** Users new to double-entry accounting in `logos` might not realize that Debits are strictly positive and Credits are strictly negative, and why this design choice was made.
**Clarification:** Added module-level documentation and struct-level docstrings with executable doc-tests in `crates/logos-core/src/domain/transaction.rs` to explicitly state the sign convention (Debits = Positive, Credits = Negative).

## 2025-02-28 - [Account Normal Balances]
**Confusion:** Users did not understand how the debit/credit sign convention mapped to specific account types (e.g. why an increase to Income is negative).
**Clarification:** Documented `AccountType::normal_balance_sign` to clearly link double-entry theory to the strict sign conventions used in `logos`.

## 2025-02-28 - [Envelope Budgeting Rollover]
**Confusion:** The meaning of `rollover_end_balance` in `BudgetMonth` wasn't obvious, especially regarding how overspending carries forward.
**Clarification:** Added module documentation and doctests demonstrating that the ending balance becomes the next month's starting balance, and negative balances (overspending) carry over until covered.

## 2025-03-01 - [UpcomingVest API Change]
**Confusion:** Users (and existing documentation) assumed `UpcomingVest` accepted a single `gross_value_cents` field, causing compilation failures.
**Clarification:** Added explicit struct documentation and doctests showing the required `avg_close_price_cents`, `units`, and `days_to_vest` fields. Added missing example to `docs/financial-planning.md`.

## 2026-03-09 - [BudgetMonth Instantiation Guide]
**Confusion:** Users were creating `BudgetMonth` instances without clear understanding of what "assigned" and "spent" meant in the context of the rollover, and no executable example existed.
**Clarification:** Added an executable doc-test example to `BudgetMonth::new` showing exactly how month rollover is calculated conceptually via an example to bridge the conceptual gap.

## 2026-03-10 - [FireSimulator Primitives]
**Confusion:** The `FireSimulator` and its configuration primitives in `logos-core::planning::fire` had dry, terse docstrings that repeated function names without explaining the underlying "why" (e.g. why risk haircut tiers exist or what a FIRE number represents practically). The module lacked a high-level explanation of its core purpose.
**Clarification:** Added a module-level `//!` narrative ("The Great Escape") explaining the goal of projecting financial independence. Upgraded struct and method docstrings to use storytelling analogies (war chest, burn rate) and added standalone, executable `## Examples` doc-tests to `FireConfig` and all public `FireSimulator` methods to demonstrate integration points clearly.

## 2026-03-15 - [AletheiaDB Start Default Paths]
**Confusion:** Users attempting to run `aletheia start` from the README's "Getting Started" section encountered confusing errors (`manifest not found at ...`) because the CLI assumes `gallifreydb` is cloned in the parent directory (`../gallifreydb`), which wasn't clearly documented.
**Clarification:** Added module-level documentation and executable doc-tests to `crates/logos-cli/src/commands/aletheia.rs`. Improved the `CliError::AletheiaStartFailed` error message to instruct users on how to resolve the issue, and updated the README to explicitly clarify the default `ALETHEIADB_MANIFEST_PATH` expectation.
