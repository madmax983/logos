#![allow(clippy::should_panic_without_expect)]

use logos_reporting::project_budget_variance;
use logos_reporting::project_cashflow;
use logos_reporting::project_net_worth;
use logos_reporting::{RegisterEntry, project_register_balance_iter};
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_cashflow_saturates_on_overflow(
        expense in 1i64..=100i64,
    ) {
        let result = project_cashflow(i64::MIN, expense);
        assert_eq!(result, i64::MIN);
    }

    #[test]
    fn project_net_worth_saturates_on_overflow(
        liabilities in 1i64..=100i64,
    ) {
        let result = project_net_worth(i64::MIN, liabilities);
        assert_eq!(result, i64::MIN);
    }

    #[test]
    fn project_budget_variance_saturates_on_overflow(
        actual in 1i64..=100i64,
    ) {
        let result = project_budget_variance(i64::MIN, actual);
        assert_eq!(result, i64::MIN);
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
