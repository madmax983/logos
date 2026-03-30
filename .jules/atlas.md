**[The Stringly-Typed Posting]**
**Tangle:** The `Posting` struct used raw `String` for its account field, creating a leaky abstraction and exposing it to stringly-typed configuration smells. `Posting::debit` and `Posting::credit` took `&str` instead of a strongly-typed domain entity.
**Blueprint:** Replaced `String` and `&str` with the `AccountId` newtype across the `Posting` struct and its public API (`debit`, `credit`, `account`). Updated all call sites (exporters, CLIs, TUI, Aletheia store) to ensure proper creation and validation of `AccountId` before passing it to double-entry structures.

**[The Stringly-Typed Envelopes and Scopes]**
**Tangle:** The `BudgetMonth`, `StoredBudgetTarget`, `StoredReconciliationRun`, and `StoredMonthClose` structs used naked `String`s for their `month_key` and account parameters (e.g. `checking_account`, `expense_account_prefix`). This exposed the storage and runtime layers to potential stringly-typed parameter mix-ups or validation drift.
**Blueprint:** Introduced a strict `MonthKey(String)` newtype in `logos-core::domain::month` that validates the `YYYY-MM` format at construction. Upgraded the core envelope models and the Aletheia storage structures to take `MonthKey` and `AccountId`. Stripped out redundant runtime empty-string validations from the storage APIs, as correctness is now guaranteed at the type-level before persistence.
**Extract Runtime to Break UI Coupling**
**Tangle:** The `logos-tui` executable crate depended directly on the `logos-cli` executable crate just to use `CliRuntime`, creating a frontend-to-frontend dependency ("The Sprawl").
**Blueprint:** Extracted `CliRuntime` (renamed to `AppRuntime`) into a new workspace crate `logos-runtime`. Updated both `logos-cli` and `logos-tui` to depend on `logos-runtime` instead, enforcing unidirectional architectural bounds.

**[The Stringly-Typed MonthKey]**
**Tangle:** The `MonthKey` domain concept was represented across the codebase as a raw `String` or `&str`, making it susceptible to stringly-typed programming errors and bypassing domain validations at the persistence and runtime layers.
**Blueprint:** Extracted the `MonthKey` concept into a strict "New Type" wrapper struct (`MonthKey(String)`) in `logos-core::domain::month`. Updated all models, storage queries, writes, and runtime CLI interactions to depend on this strict domain object. Replaced error-swallowing fallbacks at the `logos-runtime` boundaries with proper validation propagation (using `.map_err()` or `.ok()?`).
