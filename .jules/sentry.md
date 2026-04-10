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

## 2024-03-22 - [Clippy Catch]
**Learning:** `clippy::cast_possible_truncation` and `clippy::cast_lossless` catch subtle, but critical, overflow potentials when converting between integers for bounding math in property tests.
**Action:** Use `i128::from()` when casting up to guarantee lossless execution, and explicitly `#[allow(clippy::cast_possible_truncation)]` in property tests *only* when the output values are already bounded via external `if/else` logic that enforces boundaries (like `i64::MIN` or `i64::MAX`).
## 2026-03-23 - [Test All Error Paths]
**Learning:** Found an untested error branch for `AllocationPolicy::new` when percentages did not sum to 100. Always ensure custom constructor error paths are covered to prevent panics during invalid business state configuration.
**Action:** Use `cargo tarpaulin` to find untested `Err` branches in business logic files.
## 2026-04-04 - [Off-By-One Logic Mutants]
**Learning:** Checking for positive amounts using `> 0` instead of `>= 0` can be silently ignored by the test suite if there are no explicit tests mapping exactly to `0`. This leaves the system vulnerable to incorrectly evaluating an invalid value when attempting to create a ledger `Posting`.
**Action:** When filtering or excluding boundary values, write explicit test cases using precisely the rejected boundary inputs (e.g. `0%` allocations) to prove the guard is fully robust and catches `cargo-mutants` equivalent replacement cases.

## 2026-04-08 - Fix CLI Flag Value Parsing Regression with Negative Numbers
**Learning:** Fixing CLI argument parsers by rejecting any token starting with `"-"` (to prevent consuming subsequent flags as values) creates a critical regression when parsing negative amounts (like `-5000` for account balances).
**Action:** When preventing flags from being parsed as values in a CLI, specifically filter out tokens starting with `"--"` instead of `"-"` so that valid negative numeric strings can still be parsed.
