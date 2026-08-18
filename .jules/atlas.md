**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.
**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.
**[Title: MonteCarloProjector Overflow Vulnerability Fixed]**
**Tangle:** The `MonteCarloProjector::run` method accepted any `u32` for `paths`, which when given `u32::MAX`, attempted to allocate a `Vec` causing an out-of-memory (capacity overflow) panic.
**Blueprint:** Refactored `MonteCarloProjector::run` to silently cap the `paths` variable to a reasonable 10,000,000 using `.min(10_000_000)`, preventing large memory allocations. The test suite's `#[should_panic]` was removed to ensure clean completion.
**[Title: Encapsulate format module behind Facade pattern]**
**Tangle:** In `logos-core`, the `format` module implementation was exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifier from `pub mod` to `pub(crate) mod` within `logos-core` to correctly enforce the Facade pattern and encapsulate domain logic. The contents are correctly exported using `pub use format::*;`.
