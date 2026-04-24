🕸️ Tangle: The `logos-core` and `logos-store` crates exposed their internal implementation modules (`error`, `model`, `traits`, `domain`, `experimental`, `planning`) publicly (`pub mod`), leaking internal details and violating encapsulation boundaries.

📐 Blueprint: Refactored both crates to use the Facade pattern. Changed internal module visibilities to `pub(crate) mod` and explicitly re-exported only the necessary types using `pub use` at the crate roots. Updated all downstream dependencies (`logos-cli`, `logos-runtime`, `logos-store-pg`, internal tests, and fuzzers) to use the new, cleaner top-level paths.

🧱 Stability: This architectural change enforces strict separation of concerns, hides internal module organization, and provides a clean, stable public API contract for downstream consumers.

🔭 Verification: All workspace tests pass (`cargo test --workspace --all-features`). The `cargo clippy` run is warning-free. Doctests have been updated and validated.
