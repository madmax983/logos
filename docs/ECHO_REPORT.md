# 🗣️ Echo: Getting Started example is broken

## 🤦 The Confusion
I wanted to try `logos` so I literally copy-pasted the example command from the `README.md`:

```sh
cargo run -p logos-cli -- txn add --description "paycheck" --debit-account "assets:checking" --credit-account "income:salary" --amount-cents 100000
```

Instead of adding a transaction, cargo immediately threw this completely unhelpful error at my face:

```
error: failed to load manifest for workspace member `/app/crates/logos-cli`
referenced by workspace at `/app/Cargo.toml`

Caused by:
  failed to load manifest for dependency `logos-store-aletheia`

Caused by:
  failed to load manifest for dependency `aletheiadb`

Caused by:
  failed to read `/tmp/gallifreydb/Cargo.toml`

Caused by:
  No such file or directory (os error 2)
```

Why is it looking for some gallifrey doctor who database in my `/tmp` folder??

## 🕵️ The Reality
Turns out `crates/logos-store-aletheia/Cargo.toml` has a hardcoded local path dependency:

```toml
aletheiadb = { path = "/tmp/gallifreydb" }
```

The `README.md` *casually* mentions `ALETHEIADB_MANIFEST_PATH` later down in the document for running a local server, but it says this is "optional for local CLI persistence". IT IS NOT OPTIONAL! The codebase literally will not compile without that crate existing at `/tmp/gallifreydb`!

## 💡 The Fix
1. Remove the hardcoded `/tmp/gallifreydb` dependency if it's supposed to be optional or a separate feature.
2. OR: If it's mandatory, put a massive warning at the very top of `README.md` explaining how to clone `aletheiadb` to `/tmp/gallifreydb` *before* attempting to run any commands.
3. If it's on crates.io, use a normal version requirement like `aletheiadb = "0.1"`.

If I copy-paste the example and it doesn't compile, I am leaving!
