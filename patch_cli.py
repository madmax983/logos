import re

with open("crates/logos-cli/src/commands/analytics.rs", "r") as f:
    content = f.read()

# remove `use comfy_table::{Attribute, Cell, Color};` at line 255
lines = content.split('\n')
new_lines = []
for i, line in enumerate(lines):
    if i == 254 and line.strip() == "use comfy_table::{Attribute, Cell, Color};":
        pass # skip line
    else:
        new_lines.append(line)

with open("crates/logos-cli/src/commands/analytics.rs", "w") as f:
    f.write("\n".join(new_lines))

print("Patched!")
