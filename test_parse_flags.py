import re
with open("crates/logos-cli/src/args.rs", "r") as f:
    content = f.read()
print(content[:500])
