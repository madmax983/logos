use logos_core::cashflow_projector::{CashflowProjector, RecurringTemplate};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn test_cashflow_overflow(
        start_balance in (i64::MAX / 2)..i64::MAX,
        amount in (i64::MAX / 2)..i64::MAX,
    ) {
        let mut projector = CashflowProjector::new();
        projector.set_initial_balance("assets:checking", start_balance);

        projector.add_recurring_template(RecurringTemplate {
            description: "Salary".to_string(),
            amount_cents: amount,
            credit_account: "income:salary".to_string(),
            debit_account: "assets:checking".to_string(),
        });

        let _ = projector.project_balances(1);
    }
}
