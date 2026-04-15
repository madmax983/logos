# 🔭 Vantage: Spec for Income Routing Automation

👤 **User Story:**
"As a user receiving a regular paycheck, I want to automatically divide my incoming money into different checking and savings accounts based on percentage rules, so that I don't have to manually calculate and enter transfers every time I get paid."

🤔 **So What?**
Manual income allocation is tedious, error-prone, and discourages consistent saving habits. Automating this process ensures the user "pays themselves first," directly solving the business problem of low saving rates while making the CLI a low-friction tool for managing payroll deposits.

🎯 **Metric Definition:**
Success = 100% of income accurately routed according to predefined percentage rules, with all remainder cents automatically swept to ensure perfectly balanced double-entry transactions without any user intervention.

🔍 **Gap Analysis:**
Currently, users must calculate percentage splits manually and record multiple separate transfer transactions when they receive a single lump sum. Standard budgeting applications allow users to define routing templates to handle recurring income streams automatically.

✅ **Acceptance Criteria:**
- The system must accept percentage-based routing rules that sum exactly to 100%.
- The system must route a single total income amount into multiple destination accounts in a single transaction.
- The system must handle fractional cents without dropping money, sweeping any remainder into the first destination bucket to satisfy strict double-entry invariants.
- The system must return clear errors for invalid rules (e.g., percentages not equaling 100, zero/negative amounts).

🚫 **Out of Scope:**
- Integration with live bank accounts to automatically detect deposits.
- Complex routing rules based on fixed dollar amounts or tiered limits (only percentages are supported initially).
