🕸️ Tangle: The `logos-core` crate exposed its internal implementation module (`planning`) publicly (`pub mod`) and used glob imports (`pub use planning::*;`), leaking internal details and violating encapsulation boundaries. Downstream crates (`logos-cli`) and tests relied on these internal paths.

📐 Blueprint: Refactored `logos-core` to use the Facade pattern more strictly. Changed the `planning` module visibility to `pub(crate) mod` and explicitly re-exported only the necessary types using `pub use` at the crate root. Updated downstream dependencies (`logos-cli` and tests) to use the new, cleaner top-level paths.

🧱 Stability: This architectural change enforces strict separation of concerns, hides internal module organization, and provides a clean, stable public API contract for downstream consumers without relying on wildcards.

🔭 Verification: All workspace tests pass (`cargo test --workspace --all-features`). The `cargo clippy` run is warning-free. Doctests have been updated and validated.
