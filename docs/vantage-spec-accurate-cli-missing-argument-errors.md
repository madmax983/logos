# 🔭 Vantage: Spec for Accurate CLI Missing Argument Errors

👤 **User Story:**
"As a user running CLI commands, I want accurate error messages when I miss a flag entirely versus when I type the flag but forget the value, so that I can quickly fix my typo without confusion."

🤔 **So What?**
Currently, the CLI uses the same `MissingArgValue` error message for two different scenarios: missing a required flag entirely, and providing a flag but missing its value. This confuses users, wasting time trying to debug command syntax. Fixing this improves the developer experience and reduces friction during onboarding. Complexity is a cost.

🎯 **Metric Definition:**
Success = 0 users reporting confusion about missing argument CLI errors, and distinct error types returned by the CLI parser for the two scenarios.

🔍 **Gap Analysis:**
The current CLI argument parser groups "missing required argument" and "missing value for provided argument" under a single error type or message pattern. Standard CLI tools (like clap) distinguish these states.

✅ **Acceptance Criteria:**
- If a required argument is missing completely, return an error like `Missing required argument '{flag}'`.
- If a flag is provided but lacks a trailing value, return an error like `Missing value for argument '{flag}'`.
- The two error conditions must be distinct in the codebase and output.

🚫 **Out of Scope:**
- Replacing the entire CLI argument parsing library.
- Refactoring all other CLI error messages beyond these two missing argument cases.
