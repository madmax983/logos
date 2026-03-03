**[write_reconciliation_run validation gaps]**
**Mutant:** `replace < with ==` and `replace < with <=` in non-negative validations.
**Diagnosis:** [MISSING_COVERAGE] The validation paths for negative `matched_postings`, `inflow_cents` and `outflow_cents` were not covered in the test suite for both `write_reconciliation_run` and `write_reconciliation_run_and_month_close`.
**Kill Shot:** Wrote two new tests `write_reconciliation_run_fails_with_negative_values` and `write_reconciliation_run_and_month_close_fails_with_negative_values` that assert failure when passing negative parameters to those validation checks.
