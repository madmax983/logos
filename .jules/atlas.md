**[The Stringly-Typed Posting]**
**Tangle:** The `Posting` struct used raw `String` for its account field, creating a leaky abstraction and exposing it to stringly-typed configuration smells. `Posting::debit` and `Posting::credit` took `&str` instead of a strongly-typed domain entity.
**Blueprint:** Replaced `String` and `&str` with the `AccountId` newtype across the `Posting` struct and its public API (`debit`, `credit`, `account`). Updated all call sites (exporters, CLIs, TUI, Aletheia store) to ensure proper creation and validation of `AccountId` before passing it to double-entry structures.

**[The Stringly-Typed Envelopes and Scopes]**
**Tangle:** The `BudgetMonth`, `StoredBudgetTarget`, `StoredReconciliationRun`, and `StoredMonthClose` structs used naked `String`s for their `month_key` and account parameters (e.g. `checking_account`, `expense_account_prefix`). This exposed the storage and runtime layers to potential stringly-typed parameter mix-ups or validation drift.
**Blueprint:** Introduced a strict `MonthKey(String)` newtype in `logos-core::domain::month` that validates the `YYYY-MM` format at construction. Upgraded the core envelope models and the Aletheia storage structures to take `MonthKey` and `AccountId`. Stripped out redundant runtime empty-string validations from the storage APIs, as correctness is now guaranteed at the type-level before persistence.
**Extract Runtime to Break UI Coupling**
**Tangle:** The `logos-tui` executable crate depended directly on the `logos-cli` executable crate just to use `CliRuntime`, creating a frontend-to-frontend dependency ("The Sprawl").
**Blueprint:** Extracted `CliRuntime` (renamed to `AppRuntime`) into a new workspace crate `logos-runtime`. Updated both `logos-cli` and `logos-tui` to depend on `logos-runtime` instead, enforcing unidirectional architectural bounds.

**[Facade for logos-reporting]**
**Tangle:** The `logos-reporting` crate exposed all its internal implementation modules publicly (`pub mod`), leaking internal details and violating encapsulation.
**Blueprint:** Refactored `crates/logos-reporting/src/lib.rs` into a proper Facade. Changed internal modules to `pub(crate) mod` and strictly re-exported only the required public API using `pub use`.
**[Facade for logos-store-pg]**
**Tangle:** The `logos-store-pg` crate exposed its internal implementation modules (`migrate`, `schema`, `store`) publicly (`pub mod`), leaking internal database schema details and violating encapsulation.
**Blueprint:** Refactored `crates/logos-store-pg/src/lib.rs` into a Facade. Changed internal modules to `pub(crate) mod` and explicitly re-exported only the necessary types and functions (`PostgresStore`, migrations) using `pub use`.
**[Facade for logos-import]**
**Tangle:** The `logos-import` crate exposed all its internal implementation modules publicly (`pub mod csv`, `pub mod fingerprint`, `pub mod pdf`), leaking internal details and violating encapsulation.
**Blueprint:** Refactored `crates/logos-import/src/lib.rs` into a proper Facade. Changed internal modules to `pub(crate) mod` and strictly re-exported only the required public API using `pub use`.
**[Facade for logos-fetch, logos-runtime, logos-tui, logos-cli]**
**Tangle:** The `logos-fetch`, `logos-runtime`, `logos-tui`, and `logos-cli` crates exposed all their internal implementation modules publicly (e.g., `pub mod adapter;`, `pub mod app;`), leaking internal details and violating encapsulation.
**Blueprint:** Refactored `crates/logos-fetch/src/lib.rs`, `crates/logos-runtime/src/lib.rs`, `crates/logos-tui/src/lib.rs`, and `crates/logos-cli/src/lib.rs` into proper Facades. Changed internal modules to `pub(crate) mod` and strictly re-exported only the required public API using `pub use`.
**[Facade for logos-store]**
**Tangle:** The `logos-store` crate exposed its internal implementation modules (`error`, `model`, `traits`) publicly (`pub mod`), leaking internal details and violating encapsulation.
**Blueprint:** Refactored `crates/logos-store/src/lib.rs` into a Facade. Changed internal modules to `pub(crate) mod` and explicitly re-exported only the necessary types (`StoreError`, `LedgerStore`, models) using `pub use`.
**[Facade for logos-core]**
**Tangle:** The `logos-core` crate exposed its internal implementation modules (`domain`, `error`, `planning`, `experimental`) publicly (`pub mod`), leaking internal details and violating encapsulation.
**Blueprint:** Refactored `crates/logos-core/src/lib.rs` into a Facade. Changed internal modules to `pub(crate) mod` and explicitly re-exported only the necessary types using `pub use`.
**[The needless_pass_by_value optimization bug]**
**Tangle:** The `logos-store-pg` crate was hitting Clippy warnings (`needless_pass_by_value`) because mapping functions for `into_iter` were taking by-value structures (`ReconciliationRunRow`, `MonthCloseRow`) instead of references. Passing by value to `into_iter().map` allows intermediate dropping, but the clippy warning breaks build CI.
**Blueprint:** Refactored `reconciliation_run_from_row` and `month_close_from_row` to take references, properly implementing the `clippy::needless_pass_by_value` fix without reverting the `into_iter()` optimization.
