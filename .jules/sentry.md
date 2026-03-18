## 2023-10-24 - [Transaction Math Overflow Panic]
**Learning:** `Posting::credit` can panic if given `i64::MIN` due to arithmetic overflow during negation. `TransactionBuilder::build` can panic if the sum of postings overflows `i64`. Both are ticking time bombs when dealing with large values.
**Action:** Always test boundary values like `i64::MAX` and `i64::MIN` when performing arithmetic operations. Replace arithmetic operators with their `checked_*` equivalents to ensure graceful error handling.
## 2026-03-03 - [No Auto-derived Code Testing]
**Learning:** Testing auto-derived traits (like Debug, Clone) or extremely trivial code violates strict correctness bounds because it tests the compiler rather than business logic.
**Action:** Next time, carefully inspect line coverage gaps inside HTML reports to pinpoint complex logic or edge-cases (like 0, i64::MAX, negative inputs) before jumping to test basic struct traits.
## 2026-03-18 - Avoid unsafe environment manipulation in multi-threaded tests
**Learning:** Using `unsafe { std::env::set_var(...) }` in `cargo test` creates global state dependency and can cause Undefined Behavior (UB) because tests are multi-threaded by default. This violates strict safety guidelines.
**Action:** Avoid testing code that reads directly from the environment unless it can be explicitly dependency-injected. If environment tests are absolutely necessary, they must be isolated or run in single-threaded mode.
## 2026-03-18 - Ensure mocked PDF files have proper PDF structure for fallback logic
**Learning:** When testing PDF literal string extraction fallback logic (e.g. `extract_pdf_literal_strings`), providing a plain text file is insufficient. The file must contain a valid PDF header like `%PDF-1.4\n` AND the text must be enclosed in parentheses `(...)` as per the PDF specification for literal strings.
**Action:** When mocking PDF files for fallback parsing tests, wrap the text content in parentheses and prepend a valid PDF header.
