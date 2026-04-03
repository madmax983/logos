#![allow(clippy::should_panic_without_expect)]

use logos_reporting::budget_vs_actual::project_budget_variance;
use logos_reporting::cashflow::project_cashflow;
use logos_reporting::net_worth::project_net_worth;
use logos_reporting::register::{RegisterEntry, project_register_balance_iter};
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_cashflow_saturates_on_overflow(
        income in i64::MIN..=(i64::MIN + 100),
        expense in 1i64..=100i64,
    ) {
        assert_eq!(project_cashflow(i64::MIN, expense), i64::MIN);
    }

    #[test]
    fn project_net_worth_saturates_on_overflow(
        assets in i64::MIN..=(i64::MIN + 100),
        liabilities in 1i64..=100i64,
    ) {
        assert_eq!(project_net_worth(i64::MIN, liabilities), i64::MIN);
    }

    #[test]
    fn project_budget_variance_saturates_on_overflow(
        budget in i64::MIN..=(i64::MIN + 100),
        actual in 1i64..=100i64,
    ) {
        assert_eq!(project_budget_variance(i64::MIN, actual), i64::MIN);
    }

    #[test]
    fn project_register_balance_iter_saturates_on_overflow(
        opening_balance in i64::MAX..=i64::MAX,
        delta1 in 1i64..=100i64,
    ) {
        let entries = [
            RegisterEntry::new(delta1),
        ];

        let result = project_register_balance_iter(opening_balance, entries);
        assert_eq!(result, i64::MAX);
    }
}
