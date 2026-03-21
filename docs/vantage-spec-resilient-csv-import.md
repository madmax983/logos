# 🔭 Vantage: Spec for Resilient CSV Import

👤 **User Story:**
"As a user importing financial data, I want the system to safely handle malformed or unexpected CSV formats, so that my import process does not violently crash or corrupt my financial records."

💼 **Business Problem:**
The system currently crashes catastrophically (panicking on arithmetic overflow) when encountering extreme edge cases in CSV mappings (e.g., indices exceeding hardware limits). This brittle design violates the core tenet of robust financial software. If users experience sudden panics during data ingestion, they lose confidence in the system's reliability and precision. Resilience against bad input is critical to maintaining utility and user trust.

✅ **Acceptance Criteria:**
- **Success Metric:** The `parse_simple_csv_row` function (and the broader CSV import pathway) never panics due to arithmetic overflow, even when provided with pathologically large column indices (e.g., `usize::MAX`).
- The system must use safe, saturating arithmetic (e.g., `saturating_add`) or explicitly bounded calculations when computing required buffer sizes or column lengths.
- In the event of an unresolvable overflow or out-of-bounds error during parsing, the system must gracefully reject the specific row or mapping with a structured error, rather than crashing the thread.
- The `havoc_proptest` for `logos-import` must consistently pass, demonstrating that the system handles edge-case indices without panicking.

🚫 **Out of Scope:**
- Building an interactive UI to manually correct the bad CSV row.
- Parsing completely non-standard file formats that are not broadly compliant with the CSV spec.
- Automatically repairing or guessing the correct indices for a malformed configuration.
