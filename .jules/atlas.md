**[The Stringly-Typed Posting]**
**Tangle:** The `Posting` struct used raw `String` for its account field, creating a leaky abstraction and exposing it to stringly-typed configuration smells. `Posting::debit` and `Posting::credit` took `&str` instead of a strongly-typed domain entity.
**Blueprint:** Replaced `String` and `&str` with the `AccountId` newtype across the `Posting` struct and its public API (`debit`, `credit`, `account`). Updated all call sites (exporters, CLIs, TUI, Aletheia store) to ensure proper creation and validation of `AccountId` before passing it to double-entry structures.

**[The Stringly-Typed Envelopes and Scopes]**
**Tangle:** The `BudgetMonth`, `StoredBudgetTarget`, `StoredReconciliationRun`, and `StoredMonthClose` structs used naked `String`s for their `month_key` and account parameters (e.g. `checking_account`, `expense_account_prefix`). This exposed the storage and runtime layers to potential stringly-typed parameter mix-ups or validation drift.
**Blueprint:** Introduced a strict `MonthKey(String)` newtype in `logos-core::domain::month` that validates the `YYYY-MM` format at construction. Upgraded the core envelope models and the Aletheia storage structures to take `MonthKey` and `AccountId`. Stripped out redundant runtime empty-string validations from the storage APIs, as correctness is now guaranteed at the type-level before persistence.
