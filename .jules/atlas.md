**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.
**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.
**[Title: MonteCarloProjector Overflow Vulnerability Fixed]**
**Tangle:** The `MonteCarloProjector::run` method accepted any `u32` for `paths`, which when given `u32::MAX`, attempted to allocate a `Vec` causing an out-of-memory (capacity overflow) panic.
**Blueprint:** Refactored `MonteCarloProjector::run` to silently cap the `paths` variable to a reasonable 10,000,000 using `.min(10_000_000)`, preventing large memory allocations. The test suite's `#[should_panic]` was removed to ensure clean completion.
**Encapsulate format module behind Facade**
**Tangle:** The `format` module in `logos-core` was fully public (`pub mod format;`), leaking its internal structure and potentially allowing consumers to depend on its sub-module structure rather than the intended public API.
**Blueprint:** Applied the Facade pattern by restricting the module's visibility to `pub(crate) mod format;` while explicitly re-exporting its items (`pub use format::*;`) at the crate root. All workspace call sites were updated to use the root `logos_core::` path instead of `logos_core::format::`, ensuring boundaries remain clean and internal structures can change without breaking dependencies.
