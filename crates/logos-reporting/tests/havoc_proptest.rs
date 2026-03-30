#![allow(clippy::should_panic_without_expect)]

use logos_reporting::budget_vs_actual::project_budget_variance;
use logos_reporting::cashflow::project_cashflow;
use logos_reporting::net_worth::project_net_worth;
use logos_reporting::register::{project_register_balance_iter, RegisterEntry};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_budget_variance_panics_on_overflow(
        budget_cents in i64::MIN..-1,
        actual_cents in 1..i64::MAX,
    ) {
        let _ = project_budget_variance(budget_cents, actual_cents);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_cashflow_panics_on_overflow(
        income_cents in i64::MIN..-1,
        expenses_cents in 1..i64::MAX,
    ) {
        // This will panic when income_cents is very negative and expenses_cents is very positive.
        let _ = project_cashflow(income_cents, expenses_cents);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_net_worth_panics_on_overflow(
        assets in i64::MIN..-1,
        liabilities in 1..i64::MAX,
    ) {
        let _ = project_net_worth(assets, liabilities);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_register_balance_iter_panics_on_overflow(
        opening_balance in any::<i64>(),
        delta1 in any::<i64>(),
        delta2 in any::<i64>(),
    ) {
        let entries = [
            RegisterEntry::new(delta1),
            RegisterEntry::new(delta2),
        ];

        let _ = project_register_balance_iter(opening_balance, entries);
    }
}
