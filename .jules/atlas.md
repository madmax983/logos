**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.
**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.

**[Enforcing the Facade Pattern Across the Workspace]**
**Tangle:** Many internal modules across `logos-core`, `logos-cli`, `logos-fetch`, and `logos-tui` were declared as `pub mod`, violating the architectural principle of strict public APIs and leaking internal details. The compiler allowed deep, messy import paths that coupled components tightly.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. For experimental and planning modules intended to be public, we kept the module itself private (`pub(crate) mod cashflow_projector;`) but explicitly re-exported the contents at the parent level (`pub use cashflow_projector::*;`). Fixed failing doctests and integration tests by replacing internal module paths with the correct, flattened public API paths.
