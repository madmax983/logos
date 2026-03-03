## 2026-03-03 - [No Auto-derived Code Testing]
**Learning:** Testing auto-derived traits (like Debug, Clone) or extremely trivial code violates strict correctness bounds because it tests the compiler rather than business logic.
**Action:** Next time, carefully inspect line coverage gaps inside HTML reports to pinpoint complex logic or edge-cases (like 0, i64::MAX, negative inputs) before jumping to test basic struct traits.
