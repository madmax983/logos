# 🔭 Vantage: Spec for Month Autopilot

👤 **User Story:**
As an operator, I want to automatically fetch statements and reconcile the month so that I don't have to manually log in to institutions, download statements, and type opening/closing balances every month.

🤔 **So What? (Business Problem):**
The manual month-close process is tedious, error-prone, and discourages frequent reconciliation. Automating statement fetch and balance extraction removes the primary friction point in maintaining a strict double-entry ledger, transforming it from a chore into a seamless background task.

📈 **Metric Definition:**
- Success = `logos-cli month autopilot` successfully fetches statements for all configured institutions and integrates them into the import/reconcile/close pipeline without manual intervention for non-failed sources.
- Usage Metric = Number of successful automated month closures per year.

🔍 **Gap Analysis:**
Plaid and direct HTTP scraping are too brittle or privacy-invasive for this self-hosted, strict-ledger context. Existing local tools either don't automate fetching or require storing raw credentials, which is insecure. We need a headless solution that integrates securely with 1Password.

✅ **Acceptance Criteria:**
- Must execute headless browser automation via the dedicated `logos-fetch` workspace crate.
- Must support partial success across multiple institutions (one failed fetch must not poison unrelated sources).
- Must integrate fetched artifacts and extracted balances/date windows directly into the existing `month close` pipeline via `logos-cli`.
- Must persist fetch run statuses (`downloaded`, `no_new_statement`, `needs_attention`, `failed`) to audit partial failures.
- Must only allow automatic month close when fetched metadata is present and trusted.

🚫 **Out of Scope:**
- Plaid integration or undocumented direct HTTP scraping.
- Storage of raw credentials (passwords, TOTP seeds) in configuration files; only 1Password secret references (`op://...`) are allowed.
- Resolving TOTP for institutions where the seed cannot be stored in 1Password (these will be treated as `needs_attention`).
