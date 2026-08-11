**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.
**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.
**[Title: MonteCarloProjector Overflow Vulnerability Fixed]**
**Tangle:** The `MonteCarloProjector::run` method accepted any `u32` for `paths`, which when given `u32::MAX`, attempted to allocate a `Vec` causing an out-of-memory (capacity overflow) panic.
**Blueprint:** Refactored `MonteCarloProjector::run` to silently cap the `paths` variable to a reasonable 10,000,000 using `.min(10_000_000)`, preventing large memory allocations. The test suite's `#[should_panic]` was removed to ensure clean completion.

**[Tangle: The Leaky Module ]**
**Tangle:** The  module was unnecessarily exposed as a top-level  inside , creating a leaky abstraction and exposing implementation details directly to other crates.
**Blueprint:** Converted  to  and used  to safely re-export the  and  helper functions at the root of the crate. Updated all call sites in  to use the flattened  path.
**[Tangle: The Leaky Module `logos_core::format`]**
**Tangle:** The `format` module was unnecessarily exposed as a top-level `pub mod` inside `logos-core`, creating a leaky abstraction and exposing implementation details directly to other crates.
**Blueprint:** Converted `pub mod format` to `pub(crate) mod format` and used `pub use format::*;` to safely re-export the `currency` and `us_timestamp` helper functions at the root of the crate. Updated all call sites in `logos-cli` to use the flattened `logos_core::currency` path.
