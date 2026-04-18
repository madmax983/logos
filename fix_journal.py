with open(".jules/sentry.md", "r") as f:
    content = f.read()

# Add note about mutants.toml
content += """
## 2026-04-12 - [Excluding Equivalent Mutants]
**Learning:** Some mathematical boundary constraints (`amount > 0` becoming `amount >= 0`) create unviable or equivalent mutants because passing `0` to constructors (like `Posting::debit`) inherently causes domain errors that are cleanly caught and propagated via `?` up the stack.
**Action:** Added regex exclusions to `.cargo/mutants.toml` to safely ignore these known unviable permutations.
"""
with open(".jules/sentry.md", "w") as f:
    f.write(content)
