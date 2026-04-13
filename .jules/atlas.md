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

**[Facade for Workspace Libraries]**
**Tangle:** Multiple workspace libraries (`logos-fetch`, `logos-import`, `logos-store`, `logos-store-pg`, `logos-runtime`) exposed all their internal implementation modules publicly (`pub mod`), leaking internal details, polluting the public API, and violating encapsulation.
**Blueprint:** Refactored the `src/lib.rs` of these crates to use `pub mod` only where intended (e.g. traits, interfaces) and ensured that `pub use` acts as a facade, hiding internal implementation details behind clean boundaries while keeping the public API surface minimal and intention-revealing.
