with open("crates/logos-cli/src/commands/analytics.rs", "r") as f:
    content = f.read()

content = content.replace("use comfy_table::{Attribute, Cell, Color};", "")

with open("crates/logos-cli/src/commands/analytics.rs", "w") as f:
    f.write(content)

with open("crates/logos-cli/src/commands/report.rs", "r") as f:
    content = f.read()

content = content.replace("self.report.clone()", "self.report")

with open("crates/logos-cli/src/commands/report.rs", "w") as f:
    f.write(content)

print("Cli patched")
