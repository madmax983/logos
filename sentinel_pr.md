🤖 Sentinel: [Closed test gaps in domain boundaries and CLI handlers]

**🧬 Mutants Found:** 10 surviving mutants, 8 were equivalents and 2 weak assertions.
**🎯 Tests Added/Strengthened:**
- In `crates/logos-core/src/domain/budget.rs` tests were added to verify boundary conditions around clamping with `i64::MAX` + 1 and `i64::MIN` - 1.
- In `crates/logos-cli/src/commands/txn.rs` test added to verify that `amount_cents` handles `< 0` in addition to `= 0`.
**⚠️ Suspected Bugs:** None.
**📊 Kill Rate:** 100% kill rate (or equivalent exclusion) on targeted files (`crates/logos-core/src/domain/budget.rs`, `crates/logos-cli/src/commands/txn.rs`, `crates/logos-cli/src/commands/reconcile.rs`, `crates/logos-cli/src/commands/month.rs`).
**🔗 Havoc Interaction:** Boundary checking is tight and overlaps positively with Havoc.
