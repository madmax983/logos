**[Tangle: Explicit Module Re-exports within logos crates]**
**Tangle:** In `logos-cli`, `logos-tui`, `logos-core`, and `logos-fetch`, internal module implementations were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern. The only exception made was for modules explicitly re-exported (such as those imported by `logos_cli::commands::analytics`), which were properly scoped. Tests and lints were verified using `cargo test` and `cargo clippy`.
**[The Leaky Modules]**
**Tangle:** In `logos-core` and `logos-fetch`, internal module implementations (`domain`, `planning`, `experimental`, `adapters`) were exposed using `pub mod`, violating the architectural principle of strict public APIs and leaking internal details.
**Blueprint:** Updated visibility modifiers from `pub mod` to `pub(crate) mod` within these crates to correctly enforce the Facade pattern and encapsulate domain logic.
**[The Leaky Abstractions of Glob Exports]**
**Tangle:** In `logos-core/src/lib.rs`, `pub use experimental::*;` and `pub use planning::*;` bypassed the Facade pattern by exposing the internal `pub mod` declarations of 20+ experimental modules directly into the public API.
**Blueprint:** Removed the glob exports and explicitly re-exported only the structs, enums, and functions. This enforces low coupling and prevents users from depending on internal module structures while retaining access to the capabilities.
