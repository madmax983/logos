# 🗣️ Echo: Financial Planning examples are broken

**🤦 The Confusion:**
I wanted to try out the cool new Financial Planning features. I went to `docs/financial-planning.md` and copy-pasted the examples into my `main.rs`. For example, I tried to run the RSU Auto Distributor example:

```rust
use logos_core::planning::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};
// ...
```

And the FIRE Simulator example:

```rust
use logos_core::planning::fire::{FireSimulator, UpcomingVest};
// ...
```

But when I ran `cargo run`, it blew up with a bunch of scary compilation errors! It said `error[E0603]: module 'planning' is private`.

**🕵️ The Reality:**
It turns out that the `planning` module inside `logos_core` is declared as `pub(crate) mod planning;`, meaning it's private and can't be accessed from the outside like `logos_core::planning::...`. The contents are actually re-exported at the crate root (`pub use planning::*;`), so I needed to import them directly from `logos_core` (e.g., `use logos_core::fire::{FireSimulator, UpcomingVest};`).

**💡 The Fix:**
The examples in `docs/financial-planning.md` need to be updated to use the correct, public import paths (e.g., removing the `::planning::` part) so they actually compile when copy-pasted!
