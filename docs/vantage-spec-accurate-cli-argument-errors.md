# 🔭 Vantage: Spec for Accurate CLI Argument Errors

👤 **User Story:**
"As a user executing CLI commands, I want the error messages to accurately describe whether I entirely forgot a required argument or simply omitted its value, so that I can quickly correct my syntax without confusion."

🤔 **So What?**
Misleading error messages increase cognitive load and frustrate users. When the CLI tells a user they are missing a value for an argument they never even typed, the user wastes time debugging their input rather than simply adding the missing flag. Accurate, context-aware error messages build trust and make the CLI tool feel polished and reliable.

🎯 **Metric Definition:**
Success = The CLI returns distinct error messages for two separate failure modes: completely missing arguments vs missing values for provided arguments.

🔍 **Gap Analysis:**
Currently, the CLI argument parsing logic conflates two distinct user errors into a single error type. This lack of granularity forces the system to output an incorrect message regardless of whether the user typed the flag or not. Standard CLI tools correctly distinguish between a missing required argument and an incomplete argument value.

✅ **Acceptance Criteria:**
- The CLI must return an error message explicitly stating the required argument is missing when it is entirely absent from the input.
- The CLI must return an error message explicitly stating the value is missing only when the user types the flag but provides no subsequent value.
- All commands with required arguments must inherit this corrected behavior.

🚫 **Out of Scope:**
- Rewriting the entire CLI parsing library from scratch.
- Adding complex spell-checking or fuzzy-matching for mistyped argument names.
