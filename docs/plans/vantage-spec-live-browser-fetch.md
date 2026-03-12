# 🔭 Vantage: Spec for Live Browser Statement Fetch

👤 **User Story:**
"As an operator automating my monthly accounting, I want the system to actually log into my bank and download statements for me using headless browsers, so that I don't have to manually download PDFs from multiple institutions and type their balances every month."

✅ **Acceptance Criteria:**
- **Reliability:** Must successfully download the statement artifact (PDF/CSV) and extract closing balances for at least 95% of runs where credentials are valid.
- **Resilience:** If one institution fails (e.g., unexpected UI change, MFA challenge), it must isolate the failure. It must return a partial failure state (`needs_attention`) without crashing the entire month's autopilot workflow.
- **Security:** Must never store raw credentials. It must resolve all username, password, and TOTP seeds from local 1Password vault references at runtime.
- **Usability:** Must clearly communicate which institution failed and why (e.g., "MFA re-prompt required", "Invalid password") so the user can take immediate corrective action.
- **Auditability:** Every execution must persist a record of its outcome (success, failure, or needs attention) for subsequent review.

🚫 **Out of Scope:**
- Plaid or direct API integrations (we are intentionally relying on local, direct-to-institution browser automation).
- Extracting TOTP seeds from Google Authenticator (users must migrate to 1Password).
- Fixing or rewriting the institution's website if they actively block headless browsers.
- Automatically closing the month if the downloaded statement data is untrusted or missing.
