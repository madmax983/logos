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

## 2026-03-11 - [NetWorthProjector Narration]
**Confusion:** The `NetWorthProjector` lacked a narrative explaining its purpose as a timeline simulation tool, unlike the highly descriptive `FireSimulator`. Users were left to read the implementation to understand how time and monthly cash flow aggregation worked.
**Clarification:** Added a high-level "The Crystal Ball" module narrative to `crates/logos-core/src/planning/net_worth_projector.rs` explaining its purpose as the "when" to FIRE's "how much". Expanded the struct-level documentation to explain the 30-day window aggregation logic.

## 2026-03-22 - [Domain Primitives Documentation]
**Confusion:** Many core domain primitives (`AccountId`, `Category`, `Transaction`, `Correction`, `BudgetMonth`) were missing executable examples and detailed docstrings explaining their usage, leading to potential confusion about how to instantiate and use them correctly.
**Clarification:** Added comprehensive, executable `## Examples` doc-tests and descriptive `///` comments to key methods across `crates/logos-core/src/domain/`, replacing unhelpful "noise" getter docs with functional descriptions.

## 2026-03-24 - [Missing Getter Documentation]
**Confusion:** Accessor methods and counts across core and storage models (`logos-core::domain::rsu` allocation policies and `logos-store-aletheia::read` iterator methods) lacked any docstrings or examples, requiring users to read source code to verify iteration counts and structure logic.
**Clarification:** Added missing `///` comments and `## Examples` doc-tests explaining what each getter returns, avoiding noise comments ("Gets the x") and instead providing context on the stream or domain concept.
## 2026-03-24 - [Storage Accessors Documentation]
**Confusion:** The many accessor methods on `AletheiaStore` in `crates/logos-store-aletheia/src/read.rs` lacked `///` narrative documentation explaining *why* they existed (e.g., telemetry, UI point queries, deduplication) and had no executable `## Examples` doc-tests, making the API opaque to new developers.
**Clarification:** Rewrote the documentation for 25+ public accessor methods on `AletheiaStore` to strictly avoid "getter noise" (e.g. "Returns the count..."). Instead, the docs now explain the business purpose of the methods alongside executable doctests that verify their exact behavior against a new instance.

## 2026-03-25 - [RSU Reporting Module Documentation]
**Confusion:** The `logos-reporting` RSU forecasting and budgeting models (`rsu_forecast` and `rsu_budget_plan`) lacked descriptive module narratives and executable examples. Users had to reverse-engineer how scenarios (Bear/Base/Bull) mapped to strict inputs and outputs.
**Clarification:** Added module-level narratives and comprehensive executable doctests to all structs and the main projection functions (`project_rsu_forecast_summary`, `project_rsu_budget_plan`) to demonstrate instantiation and scenario sweeping semantics.

## 2026-04-02 - [Reporting Module Narratives]
**Confusion:** The `logos-reporting` crate lacked high-level module narratives (`//!`). The modules (`cashflow`, `net_worth`, `budget_vs_actual`, `register`, `rsu_forecast`) were described with dry, technical "noise" comments rather than explaining their role in the overall financial "dashboard".
**Clarification:** Rewrote the module-level documentation across `logos-reporting` to include storytelling analogies ("The Pulse", "The Scoreboard", "The Reality Check") explaining the "why" and "what" for tired developers. Added missing `## Examples` doc-tests for `RegisterEntry::delta_cents`.
