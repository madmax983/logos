🎯 Target: `logos_core::cashflow_projector` and `logos_core::net_worth_projector`
💣 Risk: Found multiple mathematical overflows when projecting future values. In `CashflowProjector` using `+=` caused panic and `NetWorthProjector` projecting more than `u16::MAX / 30` caused panic multiplying `month_index * 30`.
🧪 Strategy: Add saturating math and safely upcast `u16` month counts to `u32` before multiplication to avoid bounded overflow.
🔬 Verification: Run `cargo test --workspace` to ensure all properties pass.
