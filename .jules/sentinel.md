**[write_reconciliation_run validation gaps]**
**Mutant:** `replace < with ==` and `replace < with <=` in non-negative validations.
**Diagnosis:** [MISSING_COVERAGE] The validation paths for negative `matched_postings`, `inflow_cents` and `outflow_cents` were not covered in the test suite for both `write_reconciliation_run` and `write_reconciliation_run_and_month_close`.
**Kill Shot:** Wrote two new tests `write_reconciliation_run_fails_with_negative_values` and `write_reconciliation_run_and_month_close_fails_with_negative_values` that assert failure when passing negative parameters to those validation checks.

**Equivalent Mutants in pdf statement parser**
**Mutant:** Replaced `+` with `*` in `amount_index <= date_index + 1` and `<` with `<=` in `tokens.len() < 3` in `crates/logos-import/src/pdf.rs`.
**Diagnosis:** EQUIVALENT_MUTANT. If the mutant allows the code to proceed when it shouldn't (e.g. `amount_index <= date_index + 1`), the `memo` slice `tokens[(date_index + 1)..amount_index]` will evaluate to an empty string, causing `memo.trim().is_empty()` to catch the error anyway.
**Kill Shot:** None required. These do not alter observable behavior and should be skipped.

**Unviable Test Environment for OCR/PDF Extractor**
**Mutant:** Replaced `extract_pdf_text` and `extract_pdf_text_with_ocr` return values with `Ok(String::new())` or `Ok("xyzzy".into())`.
**Diagnosis:** MISSING_COVERAGE due to environment. The `pdftotext` and `tesseract` tools are external binaries invoked via `std::process::Command`. Without mocking the actual system calls or having a controlled environment with these tools installed and injecting mock PDFs, testing these external integration points is out of scope for pure unit tests.
**Kill Shot:** Consider abstracting the PDF extraction into a trait or injecting a command runner for proper testability in a future refactor.
