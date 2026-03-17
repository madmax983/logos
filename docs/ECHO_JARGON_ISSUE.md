# 🗣️ Echo: CLI output is full of database jargon

## 🤦 The Confusion
I ran `cargo run -p logos-cli -- txn add --description "paycheck" --debit-account "assets:checking" --credit-account "income:salary" --amount-cents 100000` from the README, and it printed `Loaded temporal adjacency index from disk`. What does that even mean? I just wanted to add a paycheck, not travel through time!

## 🕵️ The Reality
The CLI is leaking internal database terminology to the user when it loads the history index. Users do not need to know about "temporal adjacency".

## 💡 The Fix
Change the success message to something normal humans understand, like 'Loaded history index' or hide it entirely on successful runs.