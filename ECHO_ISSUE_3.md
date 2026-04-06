# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
When trying to run the Verus proofs listed in `logos-proof/README.md`, the commands look like this:
`C:\Users\markm\verus\verus.exe logos-proof\transaction_invariants.verus`
I am on a Mac (or Linux)! Why are we forcing new users to type out some developer's Windows `C:\Users\markm\` directory to run basic verifications?

**🕵️ The Reality:**
The `logos-proof/README.md` has hardcoded absolute paths pointing to a specific developer's `verus.exe` binary. This breaks copy-pasting for literally anyone else trying to onboard or verify the proofs.

**💡 The Fix:**
Change the commands in the READMEs to use a generic path (e.g., `/path/to/verus/verus` or just `verus`) and use standard forward slashes for paths. Make it clear that users need to supply their own path to the Verus executable.
