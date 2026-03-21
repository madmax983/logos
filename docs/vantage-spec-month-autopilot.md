# 🔭 Vantage: Spec for Month Autopilot

👤 **User Story:**
"As a busy professional managing my finances, I want the system to automatically fetch my monthly statements and extract balances from my various institutions, so that I can close my monthly budget without manually logging in, downloading files, and typing numbers into the CLI."

💼 **Business Problem:**
Manual statement collection and data entry is the primary friction point for users attempting to maintain strict double-entry accounting and monthly budgeting. By automating this process, we reduce time-to-value for the user, increasing the likelihood of consistent engagement and preventing users from abandoning the system due to the operational burden of the "month close." Complexity is a cost, utility is a revenue; automating the most tedious part of the workflow maximizes utility.

✅ **Acceptance Criteria:**
- **Success Metric:** 90% of successfully downloaded statements must result in accurate metadata extraction (date windows and balances) that can be seamlessly handed off to the existing `logos-cli` reconciliation and closing pipeline.
- The automation must be headless and capable of running in the background.
- It must support multiple institutions gracefully. If one institution adapter fails (e.g., due to a site change), the system must isolate the failure, continue fetching from the remaining institutions, and report partial success rather than aborting the entire process.
- The system must adhere to a strict secret model: it must never store raw credentials (usernames, passwords, or TOTP seeds). It must only store references (e.g., 1Password `op://` URIs) and resolve them securely at runtime using the local `op` CLI.
- The system must persist clear run statuses (e.g., `downloaded`, `no_new_statement`, `needs_attention`, `failed`) for each institution to provide the user with actionable feedback on what requires manual intervention.
- The output artifacts must prioritize CSV formats where available, falling back to PDF when necessary.
- The existing `logos-cli` must orchestrate the process, invoking the new `logos-fetch` workspace crate to handle the adapter execution and artifact staging.

🚫 **Out of Scope:**
- Automatic synchronization using external APIs like Plaid or undocumented direct HTTP scraping.
- Automatic closing of the month without user review when fetched metadata is missing or untrusted (automatic close is *only* permitted when trusted metadata is present).
- Browser automation logic embedded directly within the core accounting runtime (it must remain isolated in `logos-fetch`).
