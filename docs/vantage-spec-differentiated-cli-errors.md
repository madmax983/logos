# 🔭 Vantage: Spec for Differentiated CLI Errors

👤 **User Story:**
"As a CLI user, I want error messages to accurately describe whether I completely forgot a required flag or if I provided the flag but forgot its value, so that I can quickly fix my command without confusion."

🤔 **So What?**
Misleading error messages waste user time and cause frustration. When a user is told they provided a flag without a value, but they never typed the flag in the first place, they lose trust in the tool's diagnostics. Clear, differentiated errors reduce friction and make the CLI feel polished and professional.

🎯 **Metric Definition:**
Success = 100% of missing flags return a "missing flag" error, and missing values return a "missing value" error. 0 user reports of misleading missing argument errors.

🔎 **Gap Analysis:**
Currently, the CLI uses the same `MissingArgValue` error for two distinct states: completely omitting a required flag, and providing the flag without a value. Standard CLIs differentiate these states out of the box to provide clear, actionable feedback.

✅ **Acceptance Criteria:**
- When a required flag is entirely omitted, the system must return an error specifically stating the flag is missing (e.g., "Missing required argument '{flag}'").
- When a flag is provided but lacks a trailing value, the system must return an error stating the value is missing (e.g., "Missing value for argument '{flag}'").

🚫 **Out of Scope:**
- Auto-correcting typos in flag names.
- Interactive prompting for missing values.
