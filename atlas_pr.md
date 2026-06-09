🕸️ Tangle: The `logos-core`, `logos-fetch`, `logos-tui`, and `logos-cli` crates exposed their internal implementation modules (`domain`, `experimental`, `planning`, `adapters`, `ui`, `commands`) publicly (`pub mod`), leaking internal details and violating encapsulation boundaries. Some internal modules in `logos-core` (`experimental`, `planning`) were left as `pub mod` due to internal downstream crates depending on them globally via glob imports (`pub use experimental::*;`, `pub use planning::*;`), which triggers `clippy::redundant-pub-crate`.

📐 Blueprint: Refactored crates to use the Facade pattern where appropriate. Changed internal module visibilities to `pub(crate) mod` for modules like `commands`, `ui`, `adapters`, and `domain` and explicitly re-exported only the necessary types using `pub use` at the crate roots. `experimental` and `planning` in `logos-core` remained `pub mod` to comply with the glob re-exports and avoid clippy lints, although their inner structures are now correctly scoped.

🧱 Stability: This architectural change enforces strict separation of concerns, hides internal module organization, and provides a clean, stable public API contract for downstream consumers while satisfying lints.

🔭 Verification: All workspace tests pass (`cargo test --workspace --all-features`). The `cargo clippy` run is warning-free. Doctests have been updated and validated.
