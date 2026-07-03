#![allow(clippy::should_panic_without_expect)]
use logos_core::{CashflowProjector, RecurringTemplate};
use proptest::prelude::*;

proptest! {
    #[test]
    #[should_panic]
    fn project_balances_panics_on_overflow(
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

        let _ = projector.project_balances(2);
    }
}
