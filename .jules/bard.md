## 2025-02-27 - [Debits and Credits Sign Convention]
**Confusion:** Users new to double-entry accounting in `logos` might not realize that Debits are strictly positive and Credits are strictly negative, and why this design choice was made.
**Clarification:** Added module-level documentation and struct-level docstrings with executable doc-tests in `crates/logos-core/src/domain/transaction.rs` to explicitly state the sign convention (Debits = Positive, Credits = Negative).

## 2025-02-28 - [Account Normal Balances]
**Confusion:** Users did not understand how the debit/credit sign convention mapped to specific account types (e.g. why an increase to Income is negative).
**Clarification:** Documented `AccountType::normal_balance_sign` to clearly link double-entry theory to the strict sign conventions used in `logos`.

## 2025-02-28 - [Envelope Budgeting Rollover]
**Confusion:** The meaning of `rollover_end_balance` in `BudgetMonth` wasn't obvious, especially regarding how overspending carries forward.
**Clarification:** Added module documentation and doctests demonstrating that the ending balance becomes the next month's starting balance, and negative balances (overspending) carry over until covered.
