## 2025-04-07 - The Planning Black Box
**Confusion:** The `logos-core::planning` module lacked a high-level conceptual overview. Individual files (`fire`, `net_worth_projector`, `rsu_distributor`) had examples, but there was no overarching narrative connecting the "Destination" (FIRE), the "Journey" (Net Worth Projection), and the "Action" (RSU Distributor). Users were left treating it as a black box of disconnected planning primitives.
**Clarification:** Added module-level `//!` documentation to `crates/logos-core/src/planning/mod.rs` containing a unified story and an overarching executable example that uses `fire` and `net_worth_projector` together.
## 2025-04-07 - The Absolute Truths
**Confusion:** The sub-modules within the `domain` module (`transaction.rs`, `correction.rs`, `rsu.rs`) lacked clear module-level documentation explaining *why* they existed and what fundamental invariants they enforced. Users were treating them as black boxes rather than understanding their roles in the zero-trust ledger (e.g., debits equal credits, append-only history, and risk-adjusted volatility).
**Clarification:** Added module-level `//!` documentation to `transaction.rs`, `correction.rs`, and `rsu.rs` to explicitly tell the story of the strict rules of the ledger.
## 2025-04-11 - The Ghost Params
**Confusion:** The public fields of `RsuDistributorConfig` in `logos-core::planning::rsu_distributor` were completely undocumented ("Ghost Params"). Users had to infer the functional purpose of accounts like `smoothing_buffer` or `tax_reserve`, and didn't know that remainder cents are explicitly swept into the tax reserve.
**Clarification:** Added thorough `///` doc comments to all public fields in `RsuDistributorConfig` explaining their business purpose in the RSU distribution process.
