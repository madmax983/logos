**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.
**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.
