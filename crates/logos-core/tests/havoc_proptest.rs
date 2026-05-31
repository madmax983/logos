#![allow(clippy::should_panic_without_expect)]
use logos_core::cashflow_projector::{CashflowProjector, RecurringTemplate};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn test_cashflow_projector_overflow(
        balance in i64::MAX - 100..=i64::MAX,
        amount in 1i64..=100i64,
        periods in 1000u16..=2000u16
    ) {
        let mut projector = CashflowProjector::new();
        projector.set_initial_balance("assets:checking", balance);

        projector.add_recurring_template(RecurringTemplate {
            description: "Salary".to_string(),
            amount_cents: amount,
            credit_account: "income:salary".to_string(),
            debit_account: "assets:checking".to_string(),
        });

        // Should panic from integer overflow when updating balances
        let _ = projector.project_balances(periods);
    }
}
