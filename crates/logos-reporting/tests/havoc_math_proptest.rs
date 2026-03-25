use proptest::prelude::*;
use logos_reporting::{project_net_worth, project_cashflow, project_budget_variance, project_register_balance_iter, RegisterEntry, project_rsu_forecast_summary};

proptest! {
    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_net_worth_panics_on_overflow(
        _assets in i64::MIN..=i64::MIN,
        _liabilities in 1_i64..=100_i64
    ) {
        let _ = project_net_worth(_assets, _liabilities);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_cashflow_panics_on_overflow(
        _income in i64::MIN..=i64::MIN,
        _expense in 1_i64..=100_i64
    ) {
        let _ = project_cashflow(_income, _expense);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn project_budget_variance_panics_on_overflow(
        _budget in i64::MIN..=i64::MIN,
        _actual in 1_i64..=100_i64
    ) {
        let _ = project_budget_variance(_budget, _actual);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_register_balance_iter_panics_on_overflow(
        _opening_balance in i64::MAX..=i64::MAX,
        _delta in 1_i64..=100_i64
    ) {
        let entries = vec![RegisterEntry::new(_delta)];
        let _ = project_register_balance_iter(_opening_balance, entries);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn project_rsu_forecast_summary_panics_on_overflow(
        _val1 in i64::MAX..=i64::MAX,
        _val2 in 1_i64..=100_i64
    ) {
        let events = vec![_val1, _val2];
        let _ = project_rsu_forecast_summary(&events);
    }
}
