---
capture_id: cap-2026-03-11-184205-tacos
captured_at: 2026-03-11T18:42:05Z
kind: expense
amount_cents: 1284
currency: USD
merchant_memo: Tacos El Rey
from_account_hint: liabilities:amex:gold
to_account_hint:
category_hint: expenses:food:dining
status: inbox
---
Team dinner after work.

Optional notes:

- leave `to_account_hint` blank for normal card-funded expenses
- use `kind: transfer` when money moves between your own accounts
- keep `capture_id` stable forever; if the transaction changes materially after promotion or rejection, create a new note with a new id instead of editing the old one
