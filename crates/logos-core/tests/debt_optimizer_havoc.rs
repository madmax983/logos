#![cfg(feature = "nova")]
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

#[test]

fn sentinel_test_zero_balance_debt() {
    let mut optimizer = DebtOptimizer::new(100_000);
    optimizer.add_debt(Debt {
        name: "Zero Debt".to_string(),
        balance_cents: 0,
        interest_rate_pct: 10,
        min_payment_cents: 1_000,
    });

    let result = optimizer.simulate(PayoffStrategy::Avalanche);
    assert_eq!(result.total_months, 0);
    assert_eq!(result.total_interest_paid_cents, 0);
}

#[test]
fn sentinel_test_exact_payment_math() {
    let mut optimizer = DebtOptimizer::new(10_000);
    optimizer.add_debt(Debt {
        name: "Exact Math Debt".to_string(),
        balance_cents: 10_000,
        interest_rate_pct: 0, // 0 interest to test exact reduction
        min_payment_cents: 5_000,
    });

    let result = optimizer.simulate(PayoffStrategy::Avalanche);
    assert_eq!(result.total_months, 1);
    assert_eq!(result.total_interest_paid_cents, 0);
}

#[test]
fn sentinel_test_debt_needs_extra_payment() {
    let mut optimizer = DebtOptimizer::new(15_000);
    optimizer.add_debt(Debt {
        name: "Math Debt".to_string(),
        balance_cents: 20_000,
        interest_rate_pct: 0,     // 0 interest to test exact reduction
        min_payment_cents: 5_000, // pays 5_000, remaining_cash = 10_000, extra pays 10_000. Balance becomes 5_000
    });

    let result = optimizer.simulate(PayoffStrategy::Avalanche);
    assert_eq!(result.total_months, 2); // Takes 2 months
    assert_eq!(result.total_interest_paid_cents, 0);
}

#[test]
fn sentinel_test_debt_interest() {
    let mut optimizer = DebtOptimizer::new(11_000);
    optimizer.add_debt(Debt {
        name: "Interest Debt".to_string(),
        balance_cents: 120_000,
        interest_rate_pct: 10, // 10%. Monthly = 1000.
        min_payment_cents: 11_000,
    });

    let result = optimizer.simulate(PayoffStrategy::Avalanche);
    assert_eq!(result.total_months, 12);
    assert_eq!(result.total_interest_paid_cents, 6338);
}
