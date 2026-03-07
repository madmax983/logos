import re

with open("crates/logos-cli/src/runtime.rs", "r") as f:
    content = f.read()

content = content.replace(
"""        .posting(Posting::debit(AccountId::new(debit_account)?, amount_cents))
        .posting(Posting::credit(
            AccountId::new(credit_account)?,
            amount_cents,
        )?)""",
"""        .posting(Posting::debit(AccountId::new(debit_account)?, amount_cents)?)
        .posting(Posting::credit(
            AccountId::new(credit_account)?,
            amount_cents,
        )?)"""
)

# And in case there's another missing `?` somewhere:
content = re.sub(
    r'\.posting\(Posting::debit\(AccountId::new\(debit_account\)\?,\s*amount_cents\)\)',
    r'.posting(Posting::debit(AccountId::new(debit_account)?, amount_cents)?)',
    content
)

with open("crates/logos-cli/src/runtime.rs", "w") as f:
    f.write(content)
