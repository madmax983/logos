**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.
**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.

**[Facade Pattern: Encapsulate logos-core format module]**
**Tangle:** The `logos-core` crate exposed its internal formatting module publicly (`pub mod format`), leaking internal details and violating encapsulation boundaries.
**Blueprint:** Refactored `logos-core` to use the Facade pattern. Changed the `format` module visibility to `pub(crate) mod` and explicitly re-exported only the necessary functions (`currency`, `us_timestamp`) using `pub use` at the crate root. Updated all downstream dependencies (`logos-cli`, `logos-tui`) to use the new, cleaner top-level paths.
