# 🗣️ Echo: Financial Planning docs examples are broken

🤦 **The Confusion:**
I was reading the `docs/financial-planning.md` guide and decided to copy-paste the examples to try them out. I created a new binary and pasted the code for `RsuAutoDistributor`, `FireSimulator`, and `NetWorthProjector` exactly as written.
The compiler yelled at me! It said `module 'domain' is private` and `module 'planning' is private`!
It looks like this:
```text
error[E0603]: module `domain` is private
 --> src/bin/rsu_demo.rs:1:17
  |
1 | use logos_core::domain::account::AccountId;
  |                 ^^^^^^ private module
```

🕵️ **The Reality:**
It looks like the example code in the documentation is importing from internal, private modules instead of the public re-exports (e.g. `use logos_core::domain::account::AccountId` instead of `use logos_core::AccountId`). The documentation is showing internal paths that users can't actually access!

💡 **The Fix:**
Update all the `use` statements in the examples in `docs/financial-planning.md` to use the correct public paths so that if I copy-paste them, they actually compile! For example, `logos_core::planning::fire::FireSimulator` should probably just be `logos_core::fire::FireSimulator` (or whatever the public path is). And we should make sure these examples actually compile as written!
