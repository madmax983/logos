#![allow(clippy::should_panic_without_expect)]

use logos_reporting::project_budget_variance;
use logos_reporting::project_cashflow;
use logos_reporting::project_net_worth;
use logos_reporting::{RegisterEntry, project_register_balance_iter};
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_cashflow_saturates_on_overflow(
        income in i64::MAX - 10..=i64::MAX,
        expense in i64::MIN..=i64::MIN + 10,
    ) {
        let result = project_cashflow(income, expense);
        assert_eq!(result, i64::MAX);
    }

    #[test]
    fn project_cashflow_saturates_on_underflow(
        income in i64::MIN..=i64::MIN + 10,
        expense in i64::MAX - 10..=i64::MAX,
    ) {
        let result = project_cashflow(income, expense);
        assert_eq!(result, i64::MIN);
    }

    #[test]
    fn project_net_worth_saturates_on_overflow(
        assets in i64::MAX - 10..=i64::MAX,
        liabilities in i64::MIN..=i64::MIN + 10,
    ) {
        let result = project_net_worth(assets, liabilities);
        assert_eq!(result, i64::MAX);
    }

    #[test]
    fn project_net_worth_saturates_on_underflow(
        assets in i64::MIN..=i64::MIN + 10,
        liabilities in i64::MAX - 10..=i64::MAX,
    ) {
        let result = project_net_worth(assets, liabilities);
        assert_eq!(result, i64::MIN);
    }

    #[test]
    fn project_budget_variance_saturates_on_overflow(
        budget in i64::MAX - 10..=i64::MAX,
        actual in i64::MIN..=i64::MIN + 10,
    ) {
        let result = project_budget_variance(budget, actual);
        assert_eq!(result, i64::MAX);
    }

    #[test]
    fn project_budget_variance_saturates_on_underflow(
        budget in i64::MIN..=i64::MIN + 10,
        actual in i64::MAX - 10..=i64::MAX,
    ) {
        let result = project_budget_variance(budget, actual);
        assert_eq!(result, i64::MIN);
    }

    #[test]
    fn project_register_balance_iter_saturates_on_overflow(
        opening_balance in i64::MAX - 10..=i64::MAX,
        delta1 in 10i64..=100i64,
    ) {
        let entries = [
            RegisterEntry::new(delta1),
        ];

        let result = project_register_balance_iter(opening_balance, entries);
        assert_eq!(result, i64::MAX);
    }

    #[test]
    fn project_register_balance_iter_saturates_on_underflow(
        opening_balance in i64::MIN..=i64::MIN + 10,
        delta1 in -100i64..=-10i64,
    ) {
        let entries = [
            RegisterEntry::new(delta1),
        ];

        let result = project_register_balance_iter(opening_balance, entries);
        assert_eq!(result, i64::MIN);
    }
}
