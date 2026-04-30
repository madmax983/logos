# 🗣️ Echo: Getting Started example is broken

**🤦 The Confusion:**
I was trying to run the example in `docs/financial-planning.md` to simulate FIRE progress. I copy-pasted the example exactly as written:

```rust
use logos_core::planning::fire::{FireSimulator, UpcomingVest};
use logos_core::planning::net_worth_projector::NetWorthProjector;

fn main() {
    let mut sim = FireSimulator::new(500_000);
    // ...
}
```

When I ran `cargo run`, the compiler threw errors saying `module 'planning' is private`! I was stuck because the docs explicitly told me to use `logos_core::planning::fire`.

**🕵️ The Reality:**
I looked at the compiler error, and it turns out the `planning` module is declared as `pub(crate) mod planning;` in `logos-core/src/lib.rs`. It is then flattened and exported as `pub use planning::*;`. So the correct import path is actually `logos_core::fire::FireSimulator`, not `logos_core::planning::fire::FireSimulator`.

**💡 The Fix:**
Either update all the examples in `docs/financial-planning.md` to use the correct `logos_core::fire::...` import paths, or change the module visibility in `lib.rs` to `pub mod planning;` so that the code examples compile as written. The documentation shouldn't lie about the API!
