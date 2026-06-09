#![allow(clippy::should_panic_without_expect)]
use logos_core::cashflow_projector::{CashflowProjector, RecurringTemplate};
use proptest::prelude::*;

proptest! {
    #[test]
    fn project_balances_handles_overflow_gracefully(
        amount in (i64::MAX / 2 + 1)..=i64::MAX,
    ) {
        let mut projector = CashflowProjector::new();
        projector.set_initial_balance("assets:checking", amount);

        projector.add_recurring_template(RecurringTemplate {
            description: "Salary".to_string(),
            amount_cents: amount,
            credit_account: "income:salary".to_string(),
            debit_account: "assets:checking".to_string(),
        });

        let balances = projector.project_balances(2);
        assert_eq!(*balances.get("assets:checking").unwrap(), i64::MAX);
    }
}
