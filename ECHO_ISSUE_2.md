# 🗣️ Echo: README example commands fail to compile

**🤦 The Confusion:**
Tried to run the `txn add` command from the `README.md` (`cargo run -p logos-cli -- txn add --description "paycheck" --debit-account "assets:checking" --credit-account "income:salary" --amount-cents 100000`). The compiler failed with a mismatched types error in `logos-store-aletheia`.

**🕵️ The Reality:**
Turns out the `logos-store-aletheia` crate is broken and does not compile because `Posting::debit` returns a `Result`, but the code expects a direct `Posting` value inside an `Ok()` return. I shouldn't have to fix your source code just to try the tool!

**💡 The Fix:**
Fix the compile error in `crates/logos-store-aletheia/src/lib.rs` at line 2112 so the examples can actually run.