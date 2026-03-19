import re

with open("crates/logos-store-aletheia/src/read.rs", "r") as f:
    content = f.read()

# Find the end of mod tests
# It should be around line 721
lines = content.split('\n')
for i, line in enumerate(lines):
    if line.strip() == "}":
        if i > 715 and i < 730:
            tests_end = i
            break

# The tests to move start right after
tests_start = tests_end + 1

# Extract the tests
tests_content = "\n".join(lines[tests_start:])

# We want to insert the tests_content BEFORE the `tests_end` brace
# We just need to remove the #[test] blocks from the end and put them inside the mod

new_content = "\n".join(lines[:tests_end]) + "\n" + tests_content + "\n}\n"

with open("crates/logos-store-aletheia/src/read.rs", "w") as f:
    f.write(new_content)

print("Patched!")
