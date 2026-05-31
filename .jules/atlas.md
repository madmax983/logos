**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.

**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.

**[Tangle: Explicit format export in logos-core]**
**Tangle:** The `format` module in `logos-core` was exported as `pub mod format;`, which violated the architectural principle of strict public APIs and leaked internal details of the `format` module to its consumers. Previous consumers relied on things like `logos_core::format::currency`.
**Blueprint:** Updated visibility modifier of `format` to `pub(crate) mod` and directly re-exported the `currency` and `us_timestamp` functions using `pub use format::currency;` and `pub use format::us_timestamp;` at the crate root. I then updated all consumers in `logos-cli` and `logos-tui` to use the new top-level `logos_core::currency` and `logos_core::us_timestamp` functions. Verified changes via `cargo test` and `cargo clippy`.
