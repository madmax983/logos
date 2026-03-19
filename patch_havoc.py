import re

with open("crates/logos-store-aletheia/tests/havoc_kill_switch.rs", "r") as f:
    content = f.read()

# Replace #[should_panic] with #[should_panic(expected = "")]
content = content.replace("#[should_panic] // Havoc: We *expect*", '#[should_panic(expected = "")]\n// Havoc: We *expect*')

# Remove redundant clones
content = content.replace("account_checking.clone()", "account_checking")
content = content.replace("account_salary.clone()", "account_salary")
content = content.replace("txn_builder.clone()", "txn_builder")

with open("crates/logos-store-aletheia/tests/havoc_kill_switch.rs", "w") as f:
    f.write(content)

print("Havoc patched!")
