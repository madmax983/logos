# 🗣️ Echo: Misleading CLI error for missing flags

🤦 **The Confusion:**
"I ran `ledger txn add` without arguments and it failed with `Error: Missing value for argument '--description'`. I stared at my terminal confused because I didn't even type the `--description` flag in the first place! I thought it meant I typed the flag but forgot the value."

🕵️ **The Reality:**
"I did the 'Error Check' and triggered it on purpose. It turns out the CLI uses the exact same `MissingArgValue` error message for two completely different situations: when a required flag is entirely omitted, AND when the flag is provided but lacks a trailing value."

💡 **The Fix:**
"Differentiate the errors. If the flag is completely missing, return an error like `Missing required argument '{flag}'`. Keep `Missing value for argument '{flag}'` only for when the user actually types the flag but forgets the value."
