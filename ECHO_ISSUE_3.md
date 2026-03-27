# 🗣️ Echo: Verus proof example is a broken path

**🤦 The Confusion:**
I tried to run the proofs like the README said using `/path/to/verus/verus logos-proof/transaction_invariants.verus`. It threw a "No such file or directory" error. Why is the README asking me to run a nonexistent path?

**🕵️ The Reality:**
Turns out `/path/to/verus` is just a placeholder and I actually need to go download and install a completely separate tool called Verus myself, and then figure out where it installed, and then replace the path.

**💡 The Fix:**
Add a link to the Verus installation guide right before those commands, and maybe a note saying "Replace `/path/to/verus/verus` with your actual Verus installation path."
