use logos_reporting::{project_budget_variance, project_cashflow, project_net_worth};
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_cashflow_does_not_panic(
        income in i64::MIN..i64::MAX,
        expense in i64::MIN..i64::MAX
    ) {
        let _ = project_cashflow(income, expense);
    }

    #[test]
    fn project_net_worth_does_not_panic(
        assets in i64::MIN..i64::MAX,
        liabilities in i64::MIN..i64::MAX
    ) {
        let _ = project_net_worth(assets, liabilities);
    }

    #[test]
    fn project_budget_variance_does_not_panic(
        budget in i64::MIN..i64::MAX,
        actual in i64::MIN..i64::MAX
    ) {
        let _ = project_budget_variance(budget, actual);
    }
}
