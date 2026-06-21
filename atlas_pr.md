🕸️ Tangle: The `logos-core`, `logos-cli`, `logos-tui`, and `logos-fetch` crates exposed their internal implementation modules (`domain`, `experimental`, `planning`, `ui`, `commands`, `adapters`) publicly (`pub mod`), leaking internal details and violating encapsulation boundaries.

📐 Blueprint: Refactored these crates to use the Facade pattern. Changed internal module visibilities from `pub mod` to `pub(crate) mod`. Explicitly re-exported only the necessary types using `pub use` at the crate roots (e.g. `pub use planning::fire::{FireSimulator, UpcomingVest};`). Updated all downstream dependencies, tests, and fuzzers to use the new, cleaner top-level paths.

🧱 Stability: This architectural change enforces strict separation of concerns, hides internal module organization, and provides a clean, stable public API contract for downstream consumers.

🔭 Verification: All workspace tests pass (`cargo test --workspace --all-features`). The `cargo clippy` run is warning-free. Doctests have been updated and validated.
