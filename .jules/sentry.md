## 2023-10-24 - [Transaction Math Overflow Panic]
**Learning:** `Posting::credit` can panic if given `i64::MIN` due to arithmetic overflow during negation. `TransactionBuilder::build` can panic if the sum of postings overflows `i64`. Both are ticking time bombs when dealing with large values.
**Action:** Always test boundary values like `i64::MAX` and `i64::MIN` when performing arithmetic operations. Replace arithmetic operators with their `checked_*` equivalents to ensure graceful error handling.
## 2026-03-03 - [No Auto-derived Code Testing]
**Learning:** Testing auto-derived traits (like Debug, Clone) or extremely trivial code violates strict correctness bounds because it tests the compiler rather than business logic.
**Action:** Next time, carefully inspect line coverage gaps inside HTML reports to pinpoint complex logic or edge-cases (like 0, i64::MAX, negative inputs) before jumping to test basic struct traits.
