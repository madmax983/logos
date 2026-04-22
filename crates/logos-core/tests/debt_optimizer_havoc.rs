#![allow(clippy::should_panic_without_expect)]

use logos_core::experimental::debt_optimizer::{Debt, DebtOptimizer, PayoffStrategy};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn debt_optimizer_panics_on_overflow(
        balance in (i64::MAX / 2 + 1)..=i64::MAX,
        rate in 100..200_u32,
    ) {
        let mut optimizer = DebtOptimizer::new(100_000);
        optimizer.add_debt(Debt {
            name: "Massive Debt".to_string(),
            balance_cents: balance,
            interest_rate_pct: rate,
            min_payment_cents: 0,
        });

        let _ = optimizer.simulate(PayoffStrategy::Avalanche);
    }
}
