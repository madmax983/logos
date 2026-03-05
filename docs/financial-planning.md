# Financial Planning Primitives

This document covers the recently added planning components in `logos-core`.

- Module path: `crates/logos-core/src/planning/`
- Exports: `fire`, `net_worth_projector`, `rsu_distributor`
- Stability: core planning APIs (not marked experimental)

## RSU Auto Distributor

`RsuAutoDistributor` turns one RSU vest amount into a balanced double-entry transaction using a validated `AllocationPolicy`.

Primary types:

- `RsuDistributorConfig`
- `RsuAutoDistributor`
- `RsuAutoDistributor::distribute_rsu_vest(description, gross_vest_cents, policy)`

Behavior:

- Credits the configured RSU source account for the gross vest amount.
- Debits tax reserve, smoothing buffer, goals, and discretionary accounts.
- Computes smoothing/goals/discretionary amounts via integer percentage math.
- Sweeps all remainder cents to tax reserve to preserve exact balance.
- Returns `DomainError` when `TransactionBuilder` rejects invalid input (for example, empty description or unbalanced postings).

Example:

```rust
use logos_core::domain::account::AccountId;
use logos_core::domain::rsu::AllocationPolicy;
use logos_core::planning::rsu_distributor::{RsuAutoDistributor, RsuDistributorConfig};

let policy = AllocationPolicy::new(40, 20, 30, 10).expect("valid 100% allocation");
let config = RsuDistributorConfig {
    rsu_asset: AccountId::new("assets:rsu"),
    tax_reserve: AccountId::new("assets:tax"),
    smoothing_buffer: AccountId::new("assets:buffer"),
    goals: AccountId::new("assets:goals"),
    discretionary: AccountId::new("assets:checking"),
};

let distributor = RsuAutoDistributor::new(config);
let tx = distributor
    .distribute_rsu_vest("RSU vest Mar 2026", 100_000, &policy)
    .expect("balanced transaction");
assert_eq!(tx.postings().len(), 5);
```

## FIRE Simulator

`FireSimulator` estimates FIRE target, haircut-adjusted net worth, and progress percentage.

Primary types:

- `FireConfig` (`safe_withdrawal_rate_pct`, default `4`)
- `UpcomingVest`
- `FireSimulator`

Core methods:

- `FireSimulator::new(monthly_expenses_cents)`
- `set_config(FireConfig)`
- `add_assets_liabilities(assets_cents, liabilities_cents)`
- `add_upcoming_vest(UpcomingVest)`
- `fire_number_cents()`
- `safe_net_worth_cents()`
- `fire_progress_pct()`

Model details:

- FIRE number = `(monthly_expenses * 12 * 100) / safe_withdrawal_rate_pct`.
- Safe net worth = `liquid_assets - liabilities + sum(haircut-adjusted upcoming vest values)`.
- Haircuts use `HaircutTierTable::default()` unless replaced:
  - `<30 days`: 25%
  - `<=90 days`: 40%
  - `>90 days`: 55%
- Progress is clamped to `0..=100`.
- If withdrawal rate is `0`, FIRE number is treated as `i64::MAX` and progress resolves to `0`.

Example:

```rust
use logos_core::planning::fire::{FireSimulator, UpcomingVest};

let mut sim = FireSimulator::new(500_000); // $5,000/month
sim.add_assets_liabilities(20_000_000, 5_000_000); // 200k assets, 50k liabilities

sim.add_upcoming_vest(UpcomingVest {
    avg_close_price_cents: 100_000,
    units: 500,
    days_to_vest: 60,
});

assert_eq!(sim.safe_net_worth_cents(), 45_000_000); // $450k
```

## Net Worth Projector

`NetWorthProjector` simulates month-by-month net worth and milestone crossing dates.

Primary types:

- `NetWorthProjector`
- `ProjectedMonth`

Core methods:

- `NetWorthProjector::new(initial_net_worth_cents, monthly_savings_cents)`
- `set_haircut_tiers(HaircutTierTable)`
- `add_upcoming_vest(UpcomingVest)`
- `add_milestone_cents(milestone_cents)`
- `project_timeline(months) -> (Vec<ProjectedMonth>, Vec<(i64, u16)>)`

Model details:

- Each iteration represents one month and assumes a 30-day window.
- A vest contributes in the month where:
  - `days_to_vest > month_start_days`
  - `days_to_vest <= month_end_days`
- Per-month net worth updates are:
  - `previous_net_worth + monthly_savings + vested_this_month`
- Milestones are sorted ascending and recorded once at first crossing.

## Notes

- All values are integer cents (`i64` for amounts, `u16/u32` for horizon and units).
- Arithmetic uses saturating operations to reduce overflow risk.
- These are core domain/planning primitives and are not yet first-class CLI commands.
