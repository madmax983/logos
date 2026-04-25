# 🔭 Vantage: Spec for Distinct CLI Argument Errors

👤 **User Story:**
"As a user of the CLI, I want clear and distinct error messages when I omit a required flag entirely versus when I provide the flag but forget its value, so that I immediately know how to correct my command."

💼 **Business Problem:**
When users receive "Missing value for argument '--flag'" for a flag they never typed, it creates immediate confusion and distrust in the CLI's parsing logic. Misleading error messages increase cognitive load, frustrate users during onboarding, and turn simple mistakes into debugging sessions. Clear, precise error messages are essential for a usable, self-documenting CLI. Success = The CLI explicitly distinguishes between entirely absent required arguments and provided arguments that lack values.

✅ **Acceptance Criteria:**
- The CLI must return an error message like `Missing required argument '{flag}'` when a mandatory flag is completely omitted from the command.
- The CLI must return `Missing value for argument '{flag}'` only when the user types the flag but fails to provide a trailing value.
- This behavior must apply universally across all CLI commands.

🚫 **Out of Scope:**
- Redesigning the entire command parsing architecture or switching to a heavy external framework (e.g., `clap`).
- Implementing fuzzy-matching or suggestions for misspelled flags.
