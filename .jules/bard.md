## 2025-02-27 - [Debits and Credits Sign Convention]
**Confusion:** Users new to double-entry accounting in `logos` might not realize that Debits are strictly positive and Credits are strictly negative, and why this design choice was made.
**Clarification:** Added module-level documentation and struct-level docstrings with executable doc-tests in `crates/logos-core/src/domain/transaction.rs` to explicitly state the sign convention (Debits = Positive, Credits = Negative).
