**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.
**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.
**[Dependency Inversion: Presentation Layer]**
**Tangle:** `logos-cli` and `logos-tui` were tightly coupled to `logos_store_pg::PostgresStore` instead of depending on generic trait bounds, violating dependency inversion.
**Blueprint:** Refactored presentation-layer trait implementations (`ReportRuntime`, `BudgetRuntime`, `TxnPoster`, `ReconcileDataSource`, `HomeDataSource`, `BudgetDataSource`, `RegisterDataSource`) to be generic over `AppRuntime<S>` where `S: logos_store::LedgerStore`, removing direct references to `PostgresStore` and adhering to trait boundaries. Updated tests to use `MemoryStore` as generic parameter.
