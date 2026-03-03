## 2023-10-24 - [Transaction Math Overflow Panic]
**Learning:** `Posting::credit` can panic if given `i64::MIN` due to arithmetic overflow during negation. `TransactionBuilder::build` can panic if the sum of postings overflows `i64`. Both are ticking time bombs when dealing with large values.
**Action:** Always test boundary values like `i64::MAX` and `i64::MIN` when performing arithmetic operations. Replace arithmetic operators with their `checked_*` equivalents to ensure graceful error handling.
