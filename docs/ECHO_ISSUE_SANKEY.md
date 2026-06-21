# 🗣️ Echo: Jargon in analytics command help

🤦 **The Confusion:**
I ran `ledger analytics --help` to see what I could do, and I saw a command called `sankey` with the description: "Generate Mermaid Sankey diagram from current transactions". I have no idea what a "Mermaid Sankey diagram" is. It sounds like a confusing technical term, and I'm just trying to understand my personal finances.

🕵️ **The Reality:**
I did the "Slang Check" and this is pure jargon. A normal user doesn't care about the specific graphing library (Mermaid) or the formal name of the flow chart type (Sankey). They just want to see where their money is going.

💡 **The Fix:**
Rename the description to be human-readable. Something simple like "Generate a visual cashflow diagram showing where your money goes". The implementation details shouldn't be front and center in the CLI help text.
